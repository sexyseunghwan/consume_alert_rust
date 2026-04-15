# Risk Analysis Report

> Generated: 2026-04-15

## 요약

전체 위험 수: 18개 (Critical: 3, High: 5, Medium: 6, Low: 4)

---

## Critical

### [src/repository/mysql_repository.rs:85-89] `expect()`로 인한 프로세스 크래시 (OK)

- **위험 유형**: 패닉/크래시
- **코드**:
  ```rust
  let db_url: String = env::var("DATABASE_URL")
      .expect("[MysqlRepositoryImpl::new] DATABASE_URL must be set in .env");

  let db_conn: DatabaseConnection = Database::connect(db_url)
      .await
      .expect("[MysqlRepositoryImpl::new] Database connection failed");
  ```
- **이유**: `DATABASE_URL` 환경변수가 없거나 DB 연결이 실패하면 `expect()`가 즉시 프로세스를 패닉시킨다. 초기화 시점에만 호출되지만, `new()`가 `anyhow::Result`를 반환하는 구조임에도 내부에서 `?` 대신 `expect()`를 사용하고 있어 에러 전파가 불가능하다. DB 연결 재시도 로직도 없다.
- **권장 수정**: `expect()` 대신 `?` 연산자를 사용하여 에러를 상위로 전파한다.
  ```rust
  let db_url: String = env::var("DATABASE_URL")
      .context("[MysqlRepositoryImpl::new] DATABASE_URL must be set")?;
  let db_conn: DatabaseConnection = Database::connect(db_url)
      .await
      .context("[MysqlRepositoryImpl::new] Database connection failed")?;
  ```

---

### [src/repository/es_repository.rs:56-62] `expect()`로 인한 ES 초기화 시 프로세스 크래시 (OK)

- **위험 유형**: 패닉/크래시
- **코드**:
  ```rust
  let es_host: Vec<String> = env::var("ES_DB_URL")
      .expect("[EsRepositoryPub::new] 'ES_DB_URL' must be set")
      ...
  let es_id: String = env::var("ES_ID").expect("[EsRepositoryPub::new] 'ES_ID' must be set");
  let es_pw: String = env::var("ES_PW").expect("[EsRepositoryPub::new] 'ES_PW' must be set");
  ```
- **이유**: `EsRepositoryPub::new()`는 `anyhow::Result<Self>`를 반환하지만, 환경변수 로딩 3곳 모두 `expect()`를 사용한다. 환경변수 누락 시 패닉이 발생하며, 에러가 `Result`로 전파되지 않는다. 비밀번호(`ES_PW`)가 패닉 메시지로 노출될 가능성도 있다.
- **권장 수정**: 모두 `.context(...)?` 패턴으로 변경하고, 인증 정보 관련 에러 메시지에는 값 자체를 포함하지 않는다.

---

### [src/services/graph_api_service_impl.rs:27-29] `expect()`로 인한 서비스 초기화 시 패닉 (OK)

- **위험 유형**: 패닉/크래시
- **코드**:
  ```rust
  let graph_api_url: String = env::var("GRAPH_API_URL").expect(
      "[ENV file read Error][GraphApiServiceImpl -> new()] 'GRAPH_API_URL' must be set",
  );
  ```
- **이유**: `GraphApiServiceImpl::new()`는 `Result`를 반환하지 않고 `Self`를 반환한다. `GRAPH_API_URL` 환경변수가 없으면 프로세스가 즉시 패닉한다. 다른 저장소들(`MysqlRepositoryImpl`, `EsRepositoryPub`)이 `Result`를 반환하는 것과 일관성이 없으며, 에러 복구가 불가능하다.
- **권장 수정**: 반환 타입을 `anyhow::Result<Self>`로 변경하고 `?` 연산자를 사용한다.

---

## High

### [src/controller/main_controller.rs:332-337] `preprocess_string()`의 잠재적 패닉 (인덱스 범위 초과)(OK) 

- **위험 유형**: 패닉/크래시
- **코드**:
  ```rust
  fn preprocess_string(&self, delimiter: &str) -> Vec<String> {
      let args: String = self.tele_bot_service.get_input_text();
      args[2..]
          .split(delimiter)
          ...
  }
  ```

- **이유**: `args[2..]`는 바이트 인덱스 슬라이싱이다. 입력 문자열이 2바이트 미만이거나, 인덱스 2가 멀티바이트 UTF-8 문자(예: 한글)의 중간을 자를 경우 런타임 패닉이 발생한다. Telegram 메시지는 유니코드를 포함할 수 있으므로 실제 발생 가능성이 있다. 모든 명령 핸들러(`command_consumption`, `command_consumption_per_mon`, `command_consumption_per_day` 등)가 이 함수를 사용하므로 영향 범위가 넓다.
- **권장 수정**: `chars().skip(2).collect::<String>()`을 사용하거나, `args.get(2..)` 대신 `args.split_at_checked(2)`를 사용한다.
  ```rust
  args.chars().skip(2).collect::<String>()
      .split(delimiter)
      ...
  ```

---

### [src/controller/main_controller.rs:724-746] 삭제 흐름에서 Kafka 전송 성공 후 DB 삭제 실패 시 데이터 불일치

- **위험 유형**: 데이터 정합성
- **코드**:
  ```rust
  self.producer_service
      .produce_object_to_topic(produce_topic, &produce_spent_detail_info, None)
      .await
      .context("[command_delete_recent_consumption] Failed to produce Kafka message")?;

  match self
      .mysql_query_service
      .delete_spent_detail_with_transaction(spent_idx)
      .await
  {
      Ok(_) => { ... }
      Err(e) => {
          error!("...") // 에러 로깅만 하고 에러를 반환하지 않음!
      }
  }
  Ok(())
  ```
- **이유**: Kafka에 Delete 메시지를 먼저 전송한 후 MySQL에서 삭제를 시도한다. MySQL 삭제가 실패하면 `error!()` 로그만 남기고 `Ok(())`를 반환한다. 이 경우 Kafka 컨슈머(Elasticsearch 인덱서)는 문서를 삭제했지만 MySQL에는 데이터가 남아 있는 불일치 상태가 된다. 또한 삭제 실패를 사용자에게 알리지 않는다.
- **권장 수정**: MySQL 삭제 실패 시 에러를 반환하도록 수정한다. 이상적으로는 Kafka 전송과 DB 삭제의 순서를 재고하거나 보상 트랜잭션을 구현한다.
  ```rust
  Err(e) => {
      return Err(anyhow!("...{}: {:#}", spent_idx, e));
  }
  ```

---

### [src/services/process_service_impl.rs:380-383] `i32` 나눗셈에서 오버플로우 가능성

- **위험 유형**: 데이터 정합성/오버플로우
- **코드**:
  ```rust
  let spent_money: i32 = *spent_detail.spent_money();
  let spent_money_ceil: i32 = (spent_money as f32
      / *spent_detail_by_installment.installment() as f32)
      .ceil() as i32;
  ```
- **이유**: `i32::MAX`(약 21억)에 근접한 금액을 `f32`로 캐스팅하면 정밀도 손실이 발생한다. `f32`는 유효 자릿수가 약 7자리이므로 10,000,000원 이상의 금액에서 반올림 오차가 생긴다. 예: `spent_money = 10_000_001`이면 `f32` 변환 시 `10_000_000.0`으로 잘릴 수 있다. `.ceil() as i32`에서 `f32::INFINITY`나 `NaN`의 경우도 정의되지 않은 동작을 야기한다.
- **권장 수정**: `f32` 대신 `f64`를 사용하거나, 정수 나눗셈으로 처리한다.
  ```rust
  let installment = *spent_detail_by_installment.installment() as i32;
  let spent_money_ceil: i32 = (spent_money + installment - 1) / installment; // ceiling division
  ```

---

### [src/services/telebot_service_impl.rs:84-89] `thread::sleep`을 async 컨텍스트에서 사용

- **위험 유형**: 동시성/비동기 위험
- **코드**:
  ```rust
  Err(e) => {
      error!("{:?}", e);
      thread::sleep(retry_delay); // 최대 40초 * 6회 = 240초 블로킹
      attempts += 1;
  }
  ```
- **이유**: `try_send_operation()`은 `async` 함수이며, `retry_delay`는 `Duration::from_secs(40)`으로 설정된다. `thread::sleep()`은 현재 OS 스레드를 블로킹하여 최대 `40 * 6 = 240초` 동안 Tokio 워커 스레드를 점유한다. 이로 인해 동일 스레드에서 처리될 다른 async 태스크가 모두 지연된다. 특히 single-threaded 런타임이라면 전체 이벤트 루프가 멈춘다.
- **권장 수정**: `tokio::time::sleep(retry_delay).await`로 교체한다.

---

### [src/utils_modules/time_utils.rs:63-64] 월 계산 로직의 음수 모듈로 버그

- **위험 유형**: 데이터 정합성/비즈니스 로직
- **코드**:
  ```rust
  let mut new_year: i32 = naive_date.year() + (naive_date.month() as i32 + add_month - 1) / 12;
  let mut new_month: i32 = (naive_date.month() as i32 + add_month - 1) % 12 + 1;
  ```
- **이유**: Rust의 `%` 연산자는 피연산자가 음수일 때 음수 결과를 반환한다. `add_month`가 충분히 음수인 경우(예: 월이 1이고 `add_month = -13`이면 `(1 + (-13) - 1) % 12 = -13 % 12 = -1`) `new_month`가 음수가 된다. 아래의 보정 로직(`if new_month <= 0`)이 있지만, 그 이전 줄의 `new_year` 계산도 잘못될 수 있다. `(1 - 13 - 1) / 12 = -13 / 12 = -1` (Rust truncating division), 결과 `new_year`가 부정확해진다.
- **권장 수정**: `rem_euclid()`를 사용하거나 `chrono::Months`를 활용한 표준 날짜 연산으로 대체한다.
  ```rust
  // chrono의 표준 API 활용 예시
  dt.checked_add_months(Months::new(add_month.abs() as u32))
  ```

---

## Medium

### [src/services/process_service_impl.rs:258-259] Samsung 카드 오류 메시지에 NH 카드 레이블 혼용

- **위험 유형**: 데이터 정합성/버그
- **코드**:
  ```rust
  fn process_samsung_card(...) {
      let card_name: &str = split_args_vec
          .first()
          .ok_or_else(|| anyhow!("[NH Card] Price field (index 0) not found"))?; // 잘못된 레이블
      ...
      .ok_or_else(|| {
          anyhow!(
              "[NH Card] No matching payment method found for card_name: {}", // 잘못된 레이블
              card_name
          )
      })?;
  ```
- **이유**: `process_samsung_card()` 내부에서 에러 메시지의 레이블이 `"[NH Card]"`로 되어 있다. 실제로는 Samsung 카드 처리 중 발생한 에러이므로 로그 추적 시 잘못된 카드사로 오진할 수 있다. 장애 분석을 어렵게 만드는 잠재적 운영 위험이다.
- **권장 수정**: 에러 메시지를 `"[Samsung Card]"`로 수정한다.

---

### [src/services/elastic_query_service_impl.rs:124-126] 점수 계산에서 음수 곱셈 후 `i64` 변환

- **위험 유형**: 데이터 정합성
- **코드**:
  ```rust
  let keyword_weight: f64 = *consume_type.source().keyword_weight() as f64;
  let score: f64 = *consume_type.score() * -1.0 * keyword_weight;
  let score_i64: i64 = score as i64;
  ```
- **이유**: `score`가 `f64::INFINITY`, `f64::NEG_INFINITY`, 또는 `f64::NAN`인 경우 `as i64` 캐스팅은 Rust에서 정의된 동작이지만, 결과값이 `0` 또는 `i64::MIN`이 되어 잘못된 키워드 매칭이 이루어진다. `_score`가 `null`인 경우(`unwrap_or(0.0)`으로 처리됨) `score = 0.0 * -1.0 * weight = -0.0`이 되어 예상과 다른 정렬 결과가 나올 수 있다.
- **권장 수정**: `f64` 변환 전 `is_finite()` 검사를 추가한다.
  ```rust
  if !score.is_finite() {
      return Err(anyhow!("Invalid score value: {}", score));
  }
  ```

---

### [src/models/to_python_graph_line.rs:54+65-69] `i32` 누적 합산 오버플로우

- **위험 유형**: 데이터 정합성/오버플로우
- **코드**:
  ```rust
  date_consume
      .entry(elem_date)
      .and_modify(|e| *e += spent_money) // i32 덧셈
      .or_insert(spent_money);
  ...
  let mut accumulate_cost: i32 = 0;
  for cost in sorted_dates_list {
      accumulate_cost += cost; // i32 누적 합산
      consume_accumulate_list.push(accumulate_cost);
  }
  ```
- **이유**: 일별 소비 금액과 누적 합산 모두 `i32`를 사용한다. `i32::MAX`는 약 21억 원이므로, 연간 조회(`cy`)에서 연간 총 소비가 21억 원을 초과하면 debug 빌드에서 패닉, release 빌드에서 조용한 래핑(wrapping) 오버플로우가 발생한다. 하루에 같은 날짜의 여러 건이 `i32` 범위에서 합산될 때도 동일한 문제가 생긴다.
- **권장 수정**: 누적 변수를 `i64`로 변경하거나 `checked_add()`를 사용한다.

---

### [src/services/telebot_service_impl.rs:161-163] `pop()` 3번 호출로 문자 제거 시 멀티바이트 문자 경계 오류

- **위험 유형**: 패닉/데이터 손상
- **코드**:
  ```rust
  // Remove trailing ", \n"
  if !result_string.is_empty() {
      for _n in 0..3 {
          result_string.pop();
      }
  }
  ```
- **이유**: `String::pop()`은 마지막 `char`를 제거한다. 제거 대상 `", \n"`은 각각 ASCII이지만, `result_string`이 비어있거나 예상과 다른 내용으로 끝날 경우 `pop()`이 의도치 않은 문자를 제거한다. 특히 `format!()` 결과의 마지막 값이 유니코드 문자로 끝나는 경우 `pop()` 3회로 제거될 문자 경계가 달라질 수 있다. 또한 `result_string`이 정확히 1~2 문자만 있을 때 의도하지 않는 문자가 제거된다.
- **권장 수정**: `trim_end_matches(", \n")` 또는 `strip_suffix()`를 사용한다.
  ```rust
  let trimmed = result_string.trim_end_matches(|c| c == ',' || c == ' ' || c == '\n');
  result_string.truncate(trimmed.len());
  ```

---

### [src/repository/kafka_repository.rs:41] `KAFKA_BROKERS` 환경변수 누락 시 `expect()` 패닉

- **위험 유형**: 패닉/크래시
- **코드**:
  ```rust
  let kafka_brokers: String = env::var("KAFKA_BROKERS")
      .expect("[KafkaRepositoryImpl::new] 'KAFKA_BROKERS' must be set");
  ```
- **이유**: `KafkaRepositoryImpl::new()`는 `anyhow::Result<Self>`를 반환하지만, `KAFKA_BROKERS` 환경변수 로딩에서 `expect()`를 사용한다. 다른 SASL 관련 env var들도 동일 패턴이다(라인 57, 60, 63). `Result`를 반환하는 함수 내에서 `expect()`를 사용하여 에러를 전파하지 못한다.
- **권장 수정**: `?` 연산자와 `.context()`를 사용한다.

---

### [src/controller/main_controller.rs:815-818] `ctr` 명령어에서 날짜 범위 유효성 검증 미흡

- **위험 유형**: 데이터 정합성
- **코드**:
  ```rust
  let parts: Vec<&str> = args[1].split('-').collect();
  let start_date: DateTime<Utc> = parse_date_as_utc_datetime(parts[0], "%Y.%m.%d")...?;
  let end_date: DateTime<Utc> = parse_date_as_utc_datetime(parts[1], "%Y.%m.%d")...?;
  ```
- **이유**: 정규식으로 `^\d{4}\.\d{2}\.\d{2}-\d{4}\.\d{2}\.\d{2}$` 형식은 검증하지만, `start_date > end_date`인 경우를 체크하지 않는다. 예를 들어 `ctr 2024.12.01-2024.01.01` 입력 시 역방향 날짜 범위로 Elasticsearch 쿼리가 수행되어 빈 결과가 반환된다. 사용자에게 명확한 에러 메시지가 없고 빈 데이터만 표시된다.
- **권장 수정**: 파싱 후 `start_date <= end_date` 조건을 검증하고 적절한 에러 메시지를 반환한다.

---

## Low

### [src/services/redis_service_impl.rs] Redis 캐시에 TTL 없이 user_seq/room_seq 저장

- **위험 유형**: 데이터 정합성
- **코드**:
  ```rust
  // main_controller.rs:163
  self.redis_service
      .set_string(redis_key, &seq.to_string(), None) // TTL = None
      .await
  ```
- **이유**: `user_seq`와 `room_seq`를 캐싱할 때 TTL을 `None`으로 설정하여 영구 저장된다. 사용자 권한이 변경되거나 Telegram room이 삭제/재생성될 경우 Redis에는 여전히 구 캐시가 남아 있어 잘못된 `user_seq`/`room_seq`로 명령이 처리될 수 있다.
- **권장 수정**: 적절한 TTL(예: 24시간)을 설정한다.
  ```rust
  self.redis_service
      .set_string(redis_key, &seq.to_string(), Some(86400))
      .await
  ```

---

### [src/services/graph_api_service_impl.rs:59-63] HTTP 오류 응답 본문 미포함

- **위험 유형**: 에러 처리 누락
- **코드**:
  ```rust
  } else {
      Err(anyhow!(
          "[Error][post_api()] Request for '{}' failed.",
          &post_uri
      ))
  }
  ```
- **이유**: Python API 호출 실패 시 HTTP 상태 코드와 응답 본문을 에러 메시지에 포함하지 않는다. 그래프 생성 실패 원인 진단이 어렵다. `es_repository.rs`의 `delete_query()`와 비교하면, 그쪽은 상태 코드를 포함하고 있으나 `post_api()`는 누락되어 있다.
- **권장 수정**:
  ```rust
  let status = res.status();
  let error_body: String = res.text().await.unwrap_or_default();
  Err(anyhow!("[Error][post_api()] Request for '{}' failed. Status: {}, Body: {}", &post_uri, status, error_body))
  ```

---

### [src/services/process_service_impl.rs:464] `cost_map.retain(|_, v| *v >= 0)` — 0원 항목 포함

- **위험 유형**: 데이터 정합성/비즈니스 로직
- **코드**:
  ```rust
  cost_map.retain(|_, v| *v >= 0);
  ```
- **이유**: 조건이 `>= 0`이므로 0원인 카테고리도 유지된다. 이후 `get_calculate_pie_infos_from_category()`에서는 `filter(|(_, value)| **value > 0)`으로 0을 제외하는데, 두 곳의 필터 조건이 불일치한다. `retain`에서도 `> 0`으로 통일하거나, 의도적으로 0을 포함하는 이유를 문서화해야 한다. 또한 `total_cost`가 0.0일 때 `get_calculate_pie_infos_from_category()`에서 `prodt_cost as f64 / total_cost`가 `f64::INFINITY` 또는 `NaN`이 된다.
- **권장 수정**: `total_cost == 0.0` 인 경우를 early return으로 처리하고, retain 조건을 `> 0`으로 통일한다.

---

### [src/controller/main_controller.rs:897-900] 주 계산에서 날짜 오버플로우 미처리

- **위험 유형**: 패닉/데이터 손실
- **코드**:
  ```rust
  let days_to_monday: i64 = Weekday::Mon.num_days_from_monday() as i64
      - today.weekday().num_days_from_monday() as i64;
  let monday: DateTime<Utc> = today + chrono::Duration::days(days_to_monday);
  let date_end: DateTime<Utc> = monday + chrono::Duration::days(6);
  ```
- **이유**: `chrono::Duration::days()` 덧셈은 날짜가 `chrono` 지원 범위를 벗어나면 패닉을 발생시킨다. `days_to_monday`가 음수일 수 있어 과거로 이동하는 정상적인 케이스이지만, 극단적으로 큰 값이 들어올 경우(이론상 환경 이슈 등)의 안전장치가 없다. 또한 `today`가 이미 월요일인 경우 `days_to_monday = 0`이므로 정상이지만, 로직 의도가 "현재 주의 월요일"인지 "다음 주의 월요일"인지 주석이 없어 불명확하다.
- **권장 수정**: `checked_add_signed()`를 사용하여 오버플로우를 명시적으로 처리한다.
  ```rust
  let monday: DateTime<Utc> = today.checked_add_signed(chrono::Duration::days(days_to_monday))
      .ok_or_else(|| anyhow!("[command_consumption_per_week] Date overflow"))?;
  ```
