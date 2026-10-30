# 프로젝트 보안·리소스·코드 품질 검토

- 검토일: 2026-10-13, Asia/Seoul 기준
- 대상: `consume_alert_rust`, Git HEAD `b62458e`
- 방법: 실행 경로를 따라 소스 검토, 로컬 검사, 원본 날짜 함수 재현, RustSec 의존성 검사
- 변경 범위: 이 보고서만 추가. 애플리케이션 코드와 의존성은 수정하지 않음.

## 1. 우선 판단

**가장 먼저 고칠 문제는 Telegram 발신자 인증이다.** 현재 모든 메시지를 환경변수 `USER_ID`의 사용자로 처리한다. 유효한 계정 매핑이 있는 봇에 다른 사람이 메시지를 보내면 등록 사용자의 자산·소비정보를 조회하거나 지출을 추가·삭제할 수 있는 실행 경로가 있다.

운영 안정성에서도 우선 수정할 문제가 있다. 일별 조회는 하루 전체 대신 한 시점만 조회하고, 1월·12월 월별 조회는 날짜 경계 계산이 잘못된다. 주식이 없는 사용자의 자산 조회에서는 0으로 나누는 panic이 발생할 수 있다. MySQL 반영과 Kafka 이벤트 발행 사이의 실패를 복구하는 장치도 확인되지 않았다.

기본 구조에는 장점이 있다. 서비스·저장소 분리, `Arc`를 통한 클라이언트 공유, 비동기 DB 접근, DB 트랜잭션을 사용한다. 다만 인증·금융 집계·장애 복구를 검증하는 테스트가 부족해, 현재 검사가 통과해도 위 실행 경로의 문제를 검출하지 못한다.

### 우선순위 기준

| 등급 | 의미 |
| --- | --- |
| P0 | 금융정보 접근·변경 권한을 직접 우회하는 문제. 최우선 수정 |
| P1 | 주요 기능 실패, 데이터 불일치, 민감정보 노출, 서비스 중단 위험. 다음 수정 작업에서 우선 처리 |
| P2 | 특정 데이터·장애·규모에서 발생하는 정확성·성능·유지보수 문제 |

등급은 이 프로젝트의 영향을 기준으로 정한 것이다. RustSec 공지의 CVSS 점수와는 별개다. 아래의 “확인”은 코드 또는 로컬 재현으로 확인했다는 뜻이며, 실제 침해·운영 장애가 있었다는 뜻은 아니다. 배포 네트워크와 자격증명의 유효성 등 확인하지 않은 조건은 따로 명시했다.

### 발견사항 요약

| ID | 우선순위 | 발견사항 |
| --- | --- | --- |
| S01 | P0 | 실제 발신자를 확인하지 않고 고정 `USER_ID`로 인증 |
| S02 | P1 | 방 승인 상태가 권한 조회에 반영되지 않음 |
| S03 | P1 | 권한·그룹 캐시를 만료·재검증 없이 계속 신뢰 |
| S04 | P1 | Elasticsearch에 HTTP와 Basic 인증 사용 |
| S05 | P1 | 오류 로그·Redis 키에 봇 토큰 원문 포함 |
| S06 | P1, 조건부 | Git 추적 레거시 파일에 자격증명 형태의 상수 존재 |
| D01 | P1 | 잠금 파일에서 취약점 공지 일치 20건, 추가 경고 16건 |
| C01 | P1 | 일별·기간별 조회가 KST 날짜 전체를 포함하지 않음 |
| C02 | P1 | 1월·12월 월별 조회 및 전월 비교 경계 오류 |
| C03 | P1 | DB 반영과 Kafka 발행이 분리되어 부분 성공 발생 |
| C04 | P1 | 주식 투자원금이 0이면 자산 조회에서 panic |
| C05 | P1 | 자동 지출 수집에서 기본 결제수단을 제외 |
| C06 | P2 | 삼성 알림의 공백 포함 상호명 손실 |
| C07 | P2 | 연도 없는 카드 알림을 무조건 올해로 저장 |
| R01 | P1 | 그래프 API 요청에 전체 처리 timeout 없음 |
| R02 | P2 | 그래프 API 응답을 크기 제한 없이 메모리에 적재 |
| R03 | P1 | Redis 단일 연결 단절 후 재연결·DB 대체 조회 없음 |
| R04 | P2 | ES 10,000건 초과 조회가 조용히 잘림 |
| R05 | P2 | Telegram 영구 오류도 재시도하여 최소 4분 지연 |
| R06 | P2 | 여러 봇 중 뒤쪽 task의 종료 감지가 지연됨 |
| Q01 | P2 | 환경 설정 테스트 1개에 의존하고 핵심 회귀 테스트 없음 |

## 2. 보안

### S01. 실제 발신자 대신 고정 사용자 ID로 인증 — P0

**근거:** [telebot_service_impl.rs](src/services/telebot_service_impl.rs#L29) 29–50행, [main.rs](src/main.rs#L250) 250–281행, [command_resolver.rs](src/controller/main_controller/command_resolver.rs#L76) 76–80행.

`TelebotServiceImpl::new()`는 `message.from`의 발신자 ID를 읽지 않고 `AppConfig.user_id`를 저장한다. 이후 명령의 사용자·방 해석은 이 ID와 봇 토큰으로 수행한다. 응답 대상은 요청 메시지의 `message.chat.id`다. 진입점에 별도의 발신자·채팅 허용 목록 검사도 없다.

유효한 `USER_ID`/봇 토큰 매핑이 있고 제3자가 봇에 메시지를 보낼 수 있으면, 그 사람이 `my`·`cm` 등으로 금융정보를 받아 보거나 `c`·`cd`로 등록 사용자의 지출을 변경할 수 있다. 봇 토큰 원문을 알아야 하는 공격은 아니다. 봇에 메시지를 보낼 수 있는지가 조건이다.

**개선:** 실제 Telegram 발신자 ID와 승인된 내부 계정 매핑으로 인증한다. 발신자가 없는 메시지는 기본 거부하고, 공유 채팅으로 금융정보를 보낼 수 있는지도 정책에 따라 검증한다. 허용 사용자 설정을 유지한다면 실제 발신자와 비교하는 용도로 사용한다. 미등록 발신자·미허용 채팅의 조회와 변경을 모두 거부하는 테스트를 추가한다.

### S02. 방 승인 상태가 권한 조회에 반영되지 않음 — P1

**근거:** [telegram_room.rs](src/entity/telegram_room.rs#L12) 12행, [select.rs](src/services/mysql_query_service_impl/select.rs#L24) 24–28행·45–48행·65–68행.

`TELEGRAM_ROOM`에 `is_room_approved`가 있지만 인증·방·그룹 조회에는 승인 여부 필터가 없다. 따라서 이 필드를 미승인 또는 승인취소로 바꾸어도 토큰·사용자 연결이 남아 있으면 계속 허용한다. 캐시를 비워도 DB 조회 자체가 허용하는 문제다.

**개선:** 승인 상태로 접근을 통제하려는 정책이라면, 모든 관련 권한 조회에 승인 조건을 적용한다. 승인 변경 시 캐시도 무효화한다. 미승인 매핑이 존재하는 상황에서 접근이 거부되는지 검증한다.

### S03. 권한·그룹 캐시를 무기한 신뢰 — P1

**근거:** [cache_service_impl.rs](src/services/cache_service_impl.rs#L48) 48–65행·80행·124–141행·156행·200–217행·232행.

사용자·방·그룹 ID가 Redis에 있으면 즉시 반환한다. 저장 시 TTL은 모두 `None`이며, 프로젝트 안에 권한 변경과 연결된 캐시 삭제·무효화 경로가 없다. DB에서 사용자 연결을 삭제하거나 그룹을 바꿔도 이전 ID가 계속 사용될 수 있다. 그룹 이동 후 과거 그룹 조회 권한이 남는 것도 가능한 영향이다.

**개선:** 권한 변경 시 명시적 무효화, 짧은 TTL, 권한 버전 확인을 조합한다. 삭제 등 민감한 작업에는 현재 권한을 재검증한다. 외부 관리 시스템이 캐시를 지우고 있는지는 별도 확인이 필요하다. TTL 추가만으로 S02의 승인 필터 누락이 해결되지는 않는다.

### S04. Elasticsearch 통신에 HTTP와 Basic 인증 사용 — P1

**근거:** [es_repository.rs](src/repository/es_repository.rs#L69) 69–82행.

설정된 호스트마다 `http://`를 강제로 붙이고, `ES_ID`·`ES_PW`가 모두 비어 있지 않으면 Basic 인증정보를 전달한다. 애플리케이션에서 ES까지의 요청 자체에는 TLS 암호화가 없다. 해당 구간을 관측·변조할 수 있는 위치에서는 인증정보와 금융 조회 내용이 노출될 수 있다.

**개선:** 스킴을 포함한 URL을 설정으로 받고 HTTPS와 인증서 검증을 사용한다. 내부망이라도 실제 전송 구간을 확인한다. 프록시·VPN·서비스 메시 등의 별도 보호는 이번 검토에서 확인하지 않았다.

### S05. 오류 로그와 Redis 키에 봇 토큰 원문 포함 — P1

**근거:** [command_resolver.rs](src/controller/main_controller/command_resolver.rs#L88) 88–92행·134–139행·162–167행, [cache_service_impl.rs](src/services/cache_service_impl.rs#L41) 41–46행·117–122행·193–198행, [redis_repository.rs](src/repository/redis_repository.rs#L138) 138–143행·178–183행, [logger_utils.rs](src/utils_modules/logger_utils.rs#L66) 66–67행.

권한 실패 오류에 `telegram_token` 원문을 넣는다. Redis 키에도 토큰을 넣고, Redis GET/SET 실패 오류는 그 키를 그대로 포함한다. 이 오류는 파일 로그에 기록된다. 따라서 정상 등록된 봇도 Redis 장애 시 토큰을 로그에 남길 수 있다. Redis 키 열람 권한을 가진 사람도 토큰을 볼 수 있다.

**개선:** 오류에서 토큰을 제거하고 봇 ID 같은 비밀이 아닌 식별자를 사용한다. Redis 키도 비밀이 아닌 식별자로 구성한다. 요청 라이브러리 오류에 URL·API 키가 포함되는 경로도 함께 점검한다. 실제 로그에 노출된 토큰이 있다면 회전한다. 이 보고서에는 토큰 값을 기재하지 않았다.

### S06. 추적되는 레거시 파일에 자격증명 형태의 상수 존재 — P1, 조건부

**근거:** [es_multi_repository.rs](src/repository/es_multi_repository.rs#L40) 40–43행, [repository/mod.rs](src/repository/mod.rs#L5) 5행. 해당 파일은 Git 추적 대상이다.

레거시 파일에 계정·비밀번호를 포함한 URL 형태의 상수가 있다. 현재 모듈 선언이 주석 처리되어 실행 경로에는 포함되지 않는다. 자격증명의 실제 유효성과 공유 범위는 확인하지 않았다.

**개선:** 하드코딩을 제거한다. 실제 사용했거나 유효한 값이면 회전하고 Git 이력·원격 공유 범위를 확인한다. 현재 파일에서 지우는 것만으로 과거 커밋의 값이 사라지지는 않는다. 값은 이 보고서에 복사하지 않았다.

참고로 `.env`는 상위 저장소의 `.gitignore`에 의해 제외되며 현재 추적되지 않는다. 이번 검토에서 `.env`가 Git으로 노출되었다고 판단하지 않았다.

## 3. 정확성·데이터 일관성

### C01. 일별·기간별 조회가 KST 날짜 전체를 포함하지 않음 — P1

**근거:** [command_query.rs](src/controller/main_controller/command_query.rs#L235) 235–248행·271–275행, [time_utils.rs](src/utils_modules/time_utils.rs#L4) 4–15행, [elastic_query_service_impl.rs](src/services/elastic_query_service_impl.rs#L221) 221–222행.

`ct`는 시작과 끝에 동일한 UTC 자정을 전달한다. ES 요청은 시각까지 포함한 `gte/lte` 조건이다. 따라서 하루 전체가 아니라 KST 09:00:00 한 순간만 검색한다. 기간 조회도 입력 종료일의 UTC 자정까지만 조회하고, 주간·연간 및 일부 그룹 명령에도 같은 UTC/KST 경계 문제가 있다.

원본 함수를 사용하는 로컬 재현에서 `ct 2026.10.13`의 시작과 끝은 모두 `2026-10-13 00:00:00 UTC`였고, 해당 날짜의 KST 정오 지출은 범위에 포함되지 않았다.

**개선:** KST 달력에서 `[시작일 00:00, 종료일 다음날 00:00)`를 먼저 만들고 UTC로 변환한다. 종료 조건은 `lt`로 통일한다. 자산 이력의 [command_asset.rs](src/controller/main_controller/command_asset.rs#L575) 575–603행에는 이미 이 방식을 사용하는 코드가 있어 공통화할 수 있다.

### C02. 1월·12월 월별 조회 및 전월 비교 경계 오류 — P1

**근거:** [time_utils.rs](src/utils_modules/time_utils.rs#L59) 59–83행·145–159행, [command_query.rs](src/controller/main_controller/command_query.rs#L74) 74–78행, [process_service_impl.rs](src/services/process_service_impl.rs#L499) 499–506행.

`find_lastday_naivedate()`는 KST 월초를 UTC로 바꾼 결과의 달을 사용하고 일반 분기에서 `month + 2`를 계산한다. 1월은 UTC의 전년도 12월 분기로 들어가 끝이 시작보다 앞선다. 12월은 UTC 11월에 2를 더해 13월 생성에 실패한다. 전월 비교도 UTC 월말 날짜에서 월을 빼므로 KST 월초가 어긋난다.

| 원본 함수 재현 | 실제 결과 | 기대 동작 |
| --- | --- | --- |
| `cm 2026.01` | 시작 `2025-12-31 15:00 UTC`, 끝 `2025-12-31 14:59:59 UTC` | 2026년 1월 전체 |
| `cm 2026.12` | 다음 달 생성 오류 | 2026년 12월 전체 |
| `cm 2026.03`의 전월 비교 시작 | `2026-01-29 00:00 KST` | `2026-02-01 00:00 KST` |

**개선:** KST 달력에서 당월 1일·다음 달 1일·전월 1일을 계산한 뒤 각각 UTC로 변환한다. 월별 범위도 반개구간으로 처리한다. `month + 2`를 단순히 `+1`로 바꾸면 현재 UTC/KST 혼용 때문에 다른 월까지 잘못될 수 있다.

### C03. MySQL 반영과 Kafka 발행 사이에서 부분 성공 발생 — P1

**근거:** [command_consume.rs](src/controller/main_controller/command_consume.rs#L147) 147–183행·307–337행·409–439행, [mysql_repository.rs](src/repository/mysql_repository.rs#L150) 150–157행의 트랜잭션 commit 경로.

지출 등록은 DB 트랜잭션을 commit한 뒤 Kafka에 이벤트를 보낸다. 발행이 실패하면 함수는 오류로 끝나지만 DB 행은 이미 남는다. 사용자에게 완료 응답이 없어서 다시 입력하면 중복 지출이 생성될 수 있다. 삭제는 DB 삭제 후 Telegram 확인 응답을 먼저 보내므로, Telegram 전송 실패만으로도 Kafka 삭제 이벤트가 발행되지 않는다.

이벤트가 검색 인덱스 동기화에 사용되는 구조에서는 MySQL과 ES의 상태가 달라질 수 있다. 외부 consumer의 복구·재색인 정책은 이 저장소에서 확인할 수 없었다. 현재 프로젝트 안에는 이 부분 성공을 복구하는 outbox나 입력 중복 방지 경로가 없다.

**개선:** 지출 변경과 outbox 이벤트 저장을 같은 DB 트랜잭션으로 처리하고 별도 worker가 재시도하며 발행한다. 이벤트 소비는 멱등하게 처리하고, 입력에는 봇 ID·채팅 ID·메시지 ID 등으로 중복 방지 키를 둔다. Telegram 응답 실패와 금융 데이터 처리 결과를 분리한다.

### C04. 주식 투자원금이 0이면 자산 조회에서 panic — P1

**근거:** [stock_resp.rs](src/models/stock_resp.rs#L137) 137–154행, [command_asset.rs](src/controller/main_controller/command_asset.rs#L673) 673–683행.

빈 주식 목록은 “없음”으로 표시하지만 뒤에서 총 수익률을 계산할 때 투자원금으로 무조건 나눈다. 주식이 없으면 `0 / 0`이며, 모든 종목의 매입금액이 0인 경우도 해당한다. 잠긴 버전 `rust_decimal 1.36.0`의 나눗셈 구현은 0으로 나눌 때 panic한다.

로컬 `teloxide 0.12.2` 구현도 확인했다. 이 panic은 우선 해당 채팅 worker를 종료한다. 같은 채팅의 다음 메시지에서는 닫힌 worker 채널에 대한 `expect("TX is dead")`로 해당 봇의 polling task까지 종료될 수 있다. 단일 봇 구성에서는 `main` 종료로 이어진다. 운영 환경에서 실제 봇을 실행해 재현한 것은 아니다.

**개선:** 투자원금이 0일 때 ROI를 계산 불가로 표시한다. `checked_div()`를 사용하고 빈 포트폴리오·0원 원금에 대한 테스트를 추가한다.

### C05. 자동 지출 수집에서 기본 결제수단을 제외 — P1

**근거:** [command_consume.rs](src/controller/main_controller/command_consume.rs#L255) 255–257행, [select.rs](src/services/mysql_query_service_impl/select.rs#L192) 192–200행.

자동수집은 `find_user_payment_methods(user_seq, false)`를 호출한다. 구현은 `IsDefault.eq(false)`로 필터하므로 전체 카드가 아니라 기본 카드가 아닌 카드만 반환한다. 기본 카드 하나만 가진 사용자는 파서에 전달할 결제수단이 없어지고, 다른 카드가 있어도 기본 카드 알림은 처리하지 못할 수 있다.

**개선:** 전체 결제수단 조회와 기본 결제수단 조회를 별도 함수로 분리하거나 `Option<bool>` 같은 명시적인 필터를 사용한다. 자동수집에는 전체 결제수단을 전달한다.

### C06. 삼성 카드 알림의 공백 포함 상호명 손실 — P2

**근거:** [process_service_impl.rs](src/services/process_service_impl.rs#L369) 369–375행.

`10/13 12:00 스타벅스 강남점`을 공백으로 나눈 뒤 세 번째 토큰만 상호명으로 저장하므로 `스타벅스`만 남는다. `split(" ")`는 연속 공백에서 빈 토큰도 만들어 시간 파싱 실패나 빈 상호명이 발생할 수 있다. 저장된 이름과 소비 분류 정확도에 영향을 준다.

**개선:** 날짜·시간만 추출하고 나머지 상호 문자열 전체를 보존한다. NH 파서에서 처리하는 상호명 보존 방식을 참고해 파서 공통 규칙을 만든다.

### C07. 연도 없는 카드 알림을 무조건 올해로 저장 — P2

**근거:** [process_service_impl.rs](src/services/process_service_impl.rs#L84) 84–86행.

`MM/DD HH:MM`에 현재 연도를 무조건 붙인다. 1월 1일에 전날 `12/31` 알림을 전달하면 지난 연도 대신 올해 12월 31일로 저장된다. 과거 알림 재전송도 같은 문제가 있다.

**개선:** 수신 시각과 가까운 과거 날짜를 선택하는 규칙, 명시적 연도 입력, 비정상 미래 날짜 검증을 도입한다. 규칙을 테스트하려면 현재 시각을 주입할 수 있도록 한다.

## 4. 리소스·장애 복구

### R01. 그래프 API 요청에 전체 처리 timeout 없음 — P1

**근거:** [graph_api_service_impl.rs](src/services/graph_api_service_impl.rs#L12) 12–13행·61–67행.

공유 HTTP 클라이언트는 `reqwest::Client::new()`로 만든다. 로컬 `reqwest 0.11` 소스의 전체 요청 timeout 기본값은 `None`이다. 서버가 응답 헤더나 본문을 끝내지 않으면 명령이 계속 대기할 수 있다.

`teloxide 0.12.2` 기본 동작은 같은 채팅을 순차 처리하고 서로 다른 채팅을 병렬 처리한다. 한 채팅의 명령이 멈추면 후속 명령이 쌓이고, 그 채팅의 기본 64개 worker 큐가 차면 dispatcher의 전송 대기가 다른 채팅 업데이트 수신에도 영향을 줄 수 있다.

**개선:** 연결 timeout·전체 요청 timeout과 명령 전체 처리 기한을 둔다. 실제 그래프 생성 시간에 맞게 기한을 설정하고 timeout 결과를 사용자에게 전달한다.

### R02. 그래프 응답을 크기 제한 없이 메모리에 적재 — P2

**근거:** [graph_api_service_impl.rs](src/services/graph_api_service_impl.rs#L64) 64–67행, [telebot_service_impl.rs](src/services/telebot_service_impl.rs#L158) 158행.

성공 응답은 `.bytes().await?.to_vec()`, 오류 응답은 `.text()`로 전체 본문을 받는다. 크기·이미지 타입 검증이 없고 Telegram 전송 시 이미지 벡터를 복제한다. 그래프 서버의 대형·이상 응답과 여러 채팅의 동시 요청이 겹치면 메모리 사용량이 커질 수 있다.

**개선:** `Content-Length` 사전 검사와 스트리밍 수신 중 실제 크기 제한을 함께 적용한다. 이미지 타입과 오류 본문 길이도 제한한다. 이미지 바이트 공유로 불필요한 복제를 줄이고, 무거운 그래프 요청의 동시 실행 수를 제한한다. 실제 메모리 최대치는 부하 시험으로 확인해야 한다.

### R03. Redis 단일 연결 단절 후 복구 경로 없음 — P1

**근거:** [redis_repository.rs](src/repository/redis_repository.rs#L101) 101–112행·137–144행, [cache_service_impl.rs](src/services/cache_service_impl.rs#L48) 48–57행·67–87행.

단일 노드에서는 `MultiplexedConnection`을 한 번 만들고 계속 복제한다. Redis 재시작 등으로 연결이 종료되면 재연결 코드가 없다. 캐시 읽기 오류를 `?`로 반환하므로 MySQL 조회로도 넘어가지 않는다. 캐시 저장 실패 역시 정상 DB 조회의 결과를 오류로 바꾼다.

**개선:** 재연결 가능한 `redis::aio::ConnectionManager`를 사용한다. 캐시 읽기 실패 시 현재 DB 권한을 직접 검증하고, 캐시 저장 실패와 본 업무 결과를 분리한다. DB 검증까지 실패하면 접근을 거부한다. Redis 중단·복구 이후 재시작 없이 정상 처리되는지 검증한다.

참고로 로컬 `redis 1.0.3`은 기본 응답 timeout 500ms, 연결 timeout 1초가 있다. 이 항목은 무한 대기 문제가 아니라 재연결 및 캐시 실패 처리의 문제다.

### R04. ES 10,000건 초과 조회 결과가 조용히 잘림 — P2

**근거:** [elastic_query_service_impl.rs](src/services/elastic_query_service_impl.rs#L214) 214행·264–267행·306행·356–359행, [command_python_call.rs](src/controller/main_controller/command_python_call.rs#L188) 188–204행·225–235행.

방·그룹 조회 모두 `size: 10000`으로 한 응답만 처리하고 `hits.total` 확인이나 페이지 조회가 없다. 10,000건을 넘으면 ES `sum`은 전체 문서 금액을 반영하지만 상세 내역·누적 그래프·분류별 금액은 반환된 일부 문서만 반영한다. 같은 결과 안에서 합계와 그래프가 달라질 수 있다.

**개선:** 그래프·분류별 금액은 ES 날짜·분류 aggregation으로 구하고, 상세는 PIT와 `search_after` 등으로 페이지 조회한다. 먼저 잘림을 탐지해 사용자에게 표시하고 상세 출력량에도 상한·페이지 구분을 둔다.

### R05. Telegram 영구 오류도 재시도하여 최소 4분 지연 — P2

**근거:** [telebot_service_impl.rs](src/services/telebot_service_impl.rs#L73) 73–86행·135–136행·178–198행.

모든 오류에 40초 간격 재시도를 적용한다. 최초 시도와 6회 재시도 사이의 대기만 240초이며 각 네트워크 요청 시간은 별도다. 잘못된 요청·봇 차단 같은 영구 오류도 반복한다. 상세 메시지는 10개 항목씩 묶지만 문자 길이를 검사하지 않아 4,096자 한도를 넘는 동일 요청을 반복할 수 있다. 자산 요약도 길이 제한 없이 단일 메시지로 보낸다.

**개선:** 항목 수와 함께 실제 메시지 문자 길이를 기준으로 분할한다. 영구 오류는 즉시 실패시키고 `RetryAfter`는 서버가 지정한 시간을 사용한다. 일시적인 통신 오류에만 제한된 재시도를 적용한다. 길이 한도는 로컬 `teloxide-core 0.9.1`의 `send_message` 문서로 확인했다.

### R06. 뒤쪽 봇 task의 종료를 즉시 감지하지 못함 — P2

**근거:** [main.rs](src/main.rs#L292) 292–300행.

봇의 `JoinHandle`을 시작 순서대로 하나씩 기다린다. 첫 번째 봇이 계속 정상 실행하면 두 번째 이후 봇이 panic·종료되어도 결과를 확인할 차례가 오지 않는다. 정상 종료도 별도 기록하지 않는다. 일부 봇의 장애를 운영자가 놓칠 수 있다.

**개선:** `JoinSet::join_next()` 또는 `FuturesUnordered`로 완료를 즉시 수집한다. 비밀이 아닌 봇 식별자와 종료 원인을 기록하고, 제한된 재시작 또는 프로세스 종료 정책을 정한다.

## 5. 의존성과 코드 품질

### D01. RustSec 공지와 일치하는 오래된 의존성 — P1

`Cargo.lock`의 439개 패키지를 `cargo-audit 0.22.2`로 검사했다. [공식 RustSec 데이터베이스](https://github.com/RustSec/advisory-db)의 커밋 `d8325cf4f578e2df4cc5b98a30e9f46b3b88e9f0` 기준 결과는 다음과 같다.

- vulnerability 항목: **20건**
- unmaintained 경고: **6건**
- unsound 경고: **10건**

이는 **잠금 파일의 버전과 공지가 일치하는 항목 수**다. 20개의 원격 공격이 이 프로그램에서 가능하다는 의미로 해석하면 안 된다. 플랫폼·활성 기능·실제 호출 함수에 따라 영향을 구분해야 한다. 현재 Darwin arm64의 `cargo tree`도 함께 확인했다.

우선 검토할 경로는 `bytes`의 공용 네트워크 의존성, `h2 → hyper → reqwest/teloxide`, `rustls/rustls-webpki → sqlx → SeaORM`이다. `h2`의 빈 DATA 프레임 문제는 조건에 따라 메모리 사용량을 키울 수 있다. `rustls` 공지는 TLS handshake 경계 처리 문제이며 네트워크 공격자가 handshake를 임의 변조할 수 있다는 의미는 아니다. [h2 공지](https://rustsec.org/advisories/RUSTSEC-2026-0258.html), [rustls 공지](https://rustsec.org/advisories/RUSTSEC-2026-0285.html)

아래 패치 범위는 공지의 버전 범위를 그대로 기록했다. 상위 라이브러리의 버전 제약 때문에 일부는 `cargo update`만으로 해결되지 않는다. 특히 `h2 0.3`에서 공지가 제시한 `0.4.16` 이상으로 가려면 HTTP 라이브러리·Telegram 클라이언트의 호환성 검토가 필요하다.

| 잠긴 패키지 | 공지 | 공지의 패치 범위 |
| --- | --- | --- |
| `bytes 1.6.0` | [RUSTSEC-2026-0007](https://rustsec.org/advisories/RUSTSEC-2026-0007.html) | `>=1.11.1` |
| `crossbeam-channel 0.5.13` | [RUSTSEC-2025-0024](https://rustsec.org/advisories/RUSTSEC-2025-0024.html) | `>=0.5.15` |
| `crossbeam-epoch 0.9.18` | [RUSTSEC-2026-0204](https://rustsec.org/advisories/RUSTSEC-2026-0204.html) | `>=0.9.20` |
| `diesel 2.2.4` | [RUSTSEC-2026-0137](https://rustsec.org/advisories/RUSTSEC-2026-0137.html) | `>=2.3.8` |
| `diesel 2.2.4` | [RUSTSEC-2026-0136](https://rustsec.org/advisories/RUSTSEC-2026-0136.html) | `>=2.3.8` |
| `h2 0.3.26` | [RUSTSEC-2026-0258](https://rustsec.org/advisories/RUSTSEC-2026-0258.html) | `>=0.4.16` |
| `hashbrown 0.15.0` | [RUSTSEC-2024-0402](https://rustsec.org/advisories/RUSTSEC-2024-0402.html) | `>=0.15.1` |
| `idna 0.5.0` | [RUSTSEC-2024-0421](https://rustsec.org/advisories/RUSTSEC-2024-0421.html) | `>=1.0.0` |
| `openssl 0.10.64` | [RUSTSEC-2025-0004](https://rustsec.org/advisories/RUSTSEC-2025-0004.html) | `>=0.10.70` |
| `openssl 0.10.64` | [RUSTSEC-2024-0357](https://rustsec.org/advisories/RUSTSEC-2024-0357.html) | `>=0.10.66` |
| `openssl 0.10.64` | [RUSTSEC-2025-0022](https://rustsec.org/advisories/RUSTSEC-2025-0022.html) | `>=0.10.72` |
| `rkyv 0.7.45` | [RUSTSEC-2026-0001](https://rustsec.org/advisories/RUSTSEC-2026-0001.html) | `>=0.7.46, <0.8.0` 또는 `>=0.8.13` |
| `rkyv 0.7.45` | [RUSTSEC-2026-0235](https://rustsec.org/advisories/RUSTSEC-2026-0235.html) | `>=0.8.17` |
| `rsa 0.9.9` | [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071.html) | 공지 기준 패치 없음 |
| `rustls 0.23.23` | [RUSTSEC-2026-0285](https://rustsec.org/advisories/RUSTSEC-2026-0285.html) | `>=0.23.45` |
| `rustls-webpki 0.102.8` | [RUSTSEC-2026-0104](https://rustsec.org/advisories/RUSTSEC-2026-0104.html) | `>=0.103.13, <0.104.0-alpha.1` 또는 `>=0.104.0-alpha.7` |
| `rustls-webpki 0.102.8` | [RUSTSEC-2026-0098](https://rustsec.org/advisories/RUSTSEC-2026-0098.html) | `>=0.103.12, <0.104.0-alpha.1` 또는 `>=0.104.0-alpha.6` |
| `rustls-webpki 0.102.8` | [RUSTSEC-2026-0099](https://rustsec.org/advisories/RUSTSEC-2026-0099.html) | `>=0.103.12, <0.104.0-alpha.1` 또는 `>=0.104.0-alpha.6` |
| `rustls-webpki 0.102.8` | [RUSTSEC-2026-0049](https://rustsec.org/advisories/RUSTSEC-2026-0049.html) | `>=0.103.10` |
| `time 0.3.36` | [RUSTSEC-2026-0009](https://rustsec.org/advisories/RUSTSEC-2026-0009.html) | `>=0.3.47` |

적용 범위에 관한 확인 사항:

- `openssl`과 `rkyv`는 현재 Darwin arm64의 활성 의존성 트리에 나타나지 않았다. 다른 배포 대상·기능에서는 다시 확인해야 한다.
- Diesel의 위 두 vulnerability 공지는 SQLite·PostgreSQL 기능에 관한 것이다. 현재 설정은 MySQL이다. 다만 별도의 [MySQL 날짜 직렬화 unsound 경고](https://rustsec.org/advisories/RUSTSEC-2026-0134.html)도 일치했다. 현재 애플리케이션의 DB 실행 경로는 SeaORM이다.
- `hashbrown` 공지의 대상은 Borsh 직렬화이며 현재 출력된 활성 기능에는 Borsh가 없다.
- `rsa`는 SQLx MySQL 경로에 있지만 공지의 핵심은 비밀키 연산의 timing 노출이다. 해당 경로의 원격 공격 가능성을 이번 검토에서 입증하지 않았다. 패치 버전이 있다고 가정한 단순 업그레이드로 해결되지 않는다.
- `anyhow 1.0.86`의 [downcast_mut unsound 경고](https://rustsec.org/advisories/RUSTSEC-2026-0190.html)는 `>=1.0.103`에서 수정되었다. 프로젝트 소스에서 해당 함수 호출은 발견하지 않았다.

추가 경고의 전체 구성은 unmaintained의 `adler`, `dotenv`, `proc-macro-error`, `proc-macro-error2`, `rustls-pemfile` 두 버전과, unsound의 `anyhow`, Diesel 4건, `event-listener`, `lru` 2건, `rand` 두 버전이다. 유지보수 중단 경고를 취약점 건수에 합산하지 않았다.

**개선:** 사용하지 않는 직접 의존성을 먼저 정리하고, 보안 패치 가능한 버전을 갱신한다. 상위 라이브러리 변경이 필요한 항목은 별도 작업으로 검증한다. 배포 대상·활성 기능별로 재검사하고 CI에서 잠금 파일의 RustSec 검사를 지속한다. 이번 검토에서는 잠금 파일을 변경하지 않았다.

### Q01. 핵심 동작의 회귀 테스트 부족 — P2

**근거:** [config.rs](src/config.rs#L129) 129–143행. 소스 검색과 `cargo test` 실행에서 테스트는 설정 검사 1개였다.

이 테스트는 로컬 `.env` 또는 환경변수에 의존하고 전역 `OnceCell` 초기화를 사용한다. 인증·파싱·기간 경계·금융 집계·외부 장애 경로를 검증하지 않는다. 새 환경에서는 설정 부족으로 테스트 자체가 실패할 수 있다.

**개선:** 설정 파싱과 전역 초기화를 분리하고 테스트용 설정을 직접 주입한다. 기존 서비스 trait를 활용해 외부 의존성을 대체한다. 우선 다음 회귀 테스트를 추가한다.

1. 미등록 발신자, 미승인 방, 권한 취소 후 조회·변경 거부.
2. KST 00:00 경계, 1월·12월, 윤년 2월, 전월 비교, 종료일의 마지막 기록.
3. 빈 주식·0원 투자원금의 자산 조회.
4. 기본 카드만 등록된 경우, 공백 포함 상호, 연도 전환 알림.
5. DB 성공 후 Kafka 실패, Telegram 실패, 동일 메시지 재처리.
6. Redis 연결 단절 후 복구, 그래프 timeout·대형 응답, 10,000건 초과 조회.

### 작은 정리 작업

- `cargo fmt -- --check`가 5개 소스 파일의 공백·줄바꿈 차이로 실패했다. 포맷 적용과 CI 검사를 추가할 수 있다.
- [main.rs](src/main.rs#L247) 247–249행의 “봇별 순차 처리” 주석을 실제 “채팅별 순차, 채팅 간 병렬” 동작에 맞춘다.
- [config.rs](src/config.rs#L54)의 예시는 실제 이름인 `initialize()`·`get_global()` 및 필드와 불일치한다. bin 프로젝트의 현재 테스트에서는 이 문서 예시가 검증되지 않는다.
- [Cargo.toml](Cargo.toml)은 Diesel·`mysql_async`·SeaORM을 함께 선언하지만 활성 DB 호출은 SeaORM 중심이다. [schema.rs](src/schema.rs) 등 레거시 구조와 함께 사용 여부를 확인하면 빌드·네이티브 의존성·관리 부담을 줄일 수 있다.
- [.cargo/config.toml](.cargo/config.toml)의 Homebrew OpenSSL 경로는 특정 개발 환경에 묶여 있다. 배포·CI 환경에서 필요 여부를 확인해 환경별 설정으로 옮긴다.
- `common::*`의 광범위한 재수출, 문자열 `"I"`·`"D"` 이벤트 코드, 중복된 방/그룹 기간 계산은 변경 영향 파악을 어렵게 한다. 인증·기간 생성·이벤트 타입부터 공통화하는 편이 효과적이다.

## 6. 실행한 검증과 범위

| 검사 | 결과 | 해석 |
| --- | --- | --- |
| `cargo test --locked --offline` | 통과, 테스트 1개 | 로컬 환경의 설정 테스트만 검증 |
| `cargo clippy --all-targets --locked --offline -- -D warnings` | 통과 | 현재 프로젝트에 대한 Clippy 오류 없음 |
| `cargo fmt -- --check` | 실패 | 포맷 차이 존재; 소스 자동 수정은 하지 않음 |
| 공식 배포 `cargo-audit 0.22.2`, `audit --db <임시 DB 경로> --no-yanked --json` | 종료 코드 1 | vulnerability 20건, unmaintained 6건, unsound 10건 |
| 원본 날짜 함수를 사용하는 독립 Rust 재현 | 4건의 문제 확인 | 일별 경계, 1월 범위 역전, 12월 에러, 전월 시작 오류 |
| `cargo tree --locked --offline` 및 기능 출력 | 확인 완료 | 현재 플랫폼에서 의존성·활성 기능 구분 |
| `git status`, `.env` 추적·ignore 확인 | 검토 시작 시 변경 없음; `.env` 제외 | 기존 사용자 변경을 덮어쓰지 않음 |

빌드·Clippy 출력에는 의존성 `proc-macro-error2 2.0.1`의 미래 Rust 호환성 경고도 있었다. Clippy의 `-D warnings` 검사 결과와 Cargo의 이 별도 경고를 구분했다.

날짜 재현은 `/tmp/consume-alert-review-dates`의 임시 프로젝트에서 원본 `time_utils.rs`를 `#[path]`로 재사용했다. 원본 함수 출력에 대해 문제 조건을 확인하는 4개 assertion이 성공했다. 이것은 버그 재현이며 제품 회귀 테스트가 통과했다는 뜻은 아니다. 의존성은 잠금 파일과 같은 `chrono 0.4.38`, `chrono-tz 0.9.0`, `anyhow 1.0.86`, `regex 1.10.5`를 사용했다.

운영 Telegram·MySQL·Redis·Kafka·ES·그래프 서버에는 요청을 보내지 않았다. RustSec 공지·도구를 확인하기 위한 공개 인터넷 조회만 수행했다. `.env` 및 자격증명 원문은 보고서에 포함하지 않았다. 침투 테스트·부하 시험·배포 네트워크·DB의 실제 데이터와 제약조건·외부 consumer의 처리 방식·네이티브 시스템 라이브러리 취약점은 이번 범위에 포함하지 않았다. `cargo-audit`에서는 yanked 버전 조회도 제외했다.

## 7. 권장 수정 순서

1. **실제 발신자 인증과 승인 검사:** S01·S02를 고치고 미등록 사용자 접근 차단 테스트를 먼저 확보한다.
2. **자격증명과 권한 회수:** S03–S06의 캐시 무효화·로그·HTTP·하드코딩을 정리하고 실제 노출된 비밀은 회전한다.
3. **금융 기능의 정확성:** C01·C02·C04·C05부터 수정해 날짜 조회·빈 포트폴리오·기본 카드 처리를 정상화한다.
4. **데이터 일관성:** C03의 outbox와 입력 중복 방지를 구현하고 실패·재처리 검증을 추가한다.
5. **의존성 패치:** D01을 배포 대상과 호출 경로별로 분류해 갱신하고 호환성 검사를 수행한다.
6. **운영 안정성:** R01–R06의 기한·크기·조회량 제한과 연결·봇 task 복구를 적용한다.
7. **지속 검증:** Q01의 회귀 테스트, 포맷·Clippy·RustSec 검사를 CI에 연결한다.
