use linux_db;
use linux_db_dev;
use alba_test_karina_dev;


insert into SPENT_DETAIL
(
	spent_name,
    spent_money,
    spent_at,
    should_index,
    created_at,
    updated_at,
    created_by,
    updated_by,
    user_seq,
    spent_group_id,
    consume_keyword_type_id
) 
select
	prodt_name,
    prodt_money,
    DATE_SUB(timestamp, INTERVAL 9 HOUR),
    true,
    DATE_SUB(reg_dt, INTERVAL 9 HOUR),
    now(),
    'seunghwan_dev',
    'seunghwan_dev',
    1,
    1,
    1
from CONSUME_PRODT_DETAIL;


select * from COMMON_CONSUME_KEYWORD_TYPE;
select * from SPENT_DETAIL order by created_at desc;

update SPENT_DETAIL
set consume_keyword_type_id = 0
where spent_idx > 0;

CREATE TABLE SPENT_DETAIL
(
  spent_idx               BIGINT  auto_increment     NOT NULL COMMENT '지출 고유 번호',
  spent_name              VARCHAR(200) NOT NULL COMMENT '지출 이름',
  spent_money             INT          NOT NULL COMMENT '지출금',
  spent_at                DATETIME     NOT NULL COMMENT '지출 시각',
  should_index            BOOLEAN      NOT NULL COMMENT '색인 대상 여부',
  created_at              DATETIME     NOT NULL COMMENT '생성 시각',
  updated_at              DATETIME     NULL     COMMENT '수정 시각',
  created_by              VARCHAR(100) NOT NULL COMMENT '생성자',
  updated_by              VARCHAR(100) NULL     COMMENT '수정자',
  user_seq                BIGINT       NOT NULL COMMENT '유저 식별번호',
  spent_group_id          BIGINT       NOT NULL COMMENT '지출 그룹 고유번호',
  consume_keyword_type_id BIGINT       NOT NULL COMMENT '대표 키워드 타입 아이디',
  PRIMARY KEY (spent_idx)
) ENGINE=InnoDB COMMENT '지출 내역 테이블';

create index idx_spent_detail_user_seq on SPENT_DETAIL (user_seq);
create index idx_spent_detail_spent_group_id on SPENT_DETAIL (spent_group_id);
create index idx_spent_detail_consume_keyword_type_id on SPENT_DETAIL (consume_keyword_type_id);

CREATE TABLE SPENT_DETAIL
(
  spent_idx      BIGINT   auto_increment    NOT NULL COMMENT '지출 고유 번호',
  spent_name     VARCHAR(200) NOT NULL COMMENT '지출 이름',
  spent_money    INT          NOT NULL COMMENT '지출금',
  spent_at       DATETIME     NOT NULL COMMENT '지출 시각',
  should_index   BOOLEAN      NOT NULL COMMENT '색인 대상 여부',
  created_at     DATETIME     NOT NULL COMMENT '생성 시각',
  updated_at     DATETIME     NULL     COMMENT '수정 시각',
  created_by     VARCHAR(100) NOT NULL COMMENT '생성자',
  updated_by     VARCHAR(100) NULL     COMMENT '수정자',
  user_seq       BIGINT       NOT NULL COMMENT '유저 식별번호',
  spent_group_id BIGINT       NOT NULL COMMENT '지출 그룹 고유번호',
  PRIMARY KEY (spent_idx)
) ENGINE=InnoDB COMMENT '지출 내역 테이블';

create index idx_spent_detail_user_seq on SPENT_DETAIL (user_seq);
create index idx_spent_detail_spent_group_id on SPENT_DETAIL (spent_group_id);

desc SPENT_DETAIL;



select * from COMMON_CONSUME_KEYWORD_TYPE;

select * from COMMON_CONSUME_PRODT_KEYWORD;

select 
	consume_keyword_type
from COMMON_CONSUME_KEYWORD_TYPE
where consume_keyword_type_id = 1;

select count(*) from COMMON_CONSUME_PRODT_KEYWORD;

select * from SPENT_GROUP_INFO;
select * from COMMON_CONSUME_PRODT_KEYWORD;

select
*
from SPENT_DETAIL sd
inner join COMMON_CONSUME_PRODT_KEYWORD cp on sd.consume_keyword_id = cp.consume_keyword_id
inner join COMMON_CONSUME_KEYWORD_TYPE ct on cp.consume_keyword_type_id = ct.consume_keyword_type_id;


select
*
from SPENT_DETAIL order by created_at desc;

select
	sd.spent_idx,
	sd.spent_name,
    sd.spent_money,
    sd.spent_at,
    sd.created_at,
    sd.user_seq,
    ct.consume_keyword_type_id,
    ct.consume_keyword_type,
    t.room_seq
from SPENT_DETAIL sd
inner join COMMON_CONSUME_KEYWORD_TYPE ct on ct.consume_keyword_type_id = sd.consume_keyword_type_id
inner join USERS u on u.user_seq = sd.user_seq
left join TELEGRAM_ROOM t on u.user_seq = t.user_seq 
where sd.should_index = 1
and t.is_room_approved = true
order by sd.created_at desc;


select * from COMMON_CONSUME_KEYWORD_TYPE;

select * from SPENT_DETAIL order by created_at desc; 
-- 15

select * from COMMON_CONSUME_PRODT_KEYWORD;
select * from COMMON_CONSUME_KEYWORD_TYPE;





select * from COMMON_CONSUME_PRODT_KEYWORD;
select * from COMMON_CONSUME_KEYWORD_TYPE;

desc COMMON_CONSUME_PRODT_KEYWORD;

show tables;

select count(*) from SPENT_DETAIL;

select * from SPENT_DETAIL order by created_at desc;

select * from TELEGRAM_ROOM;

select *
from USERS u
inner join TELEGRAM_ROOM t on u.user_seq = t.user_seq;

select * from USERS;

select * from TELEGRAM_ROOM;

select * from COMMON_CONSUME_KEYWORD_TYPE where consume_keyword_type_id = 15; 
#where 

select * from SPENT_DETAIL order by created_by desc;
#order by 

select * from COMMON_CONSUME_PRODT_KEYWORD;

desc TELEGRAM_ROOM;

desc USERS;

select * from COMMON_CONSUME_PRODT_KEYWORD;
select * from COMMON_CONSUME_KEYWORD_TYPE;

alter table SPENT_DETAIL add column consume_keyword_id bigint not null default 0;

select * from SPENT_DETAIL;

select * from USERS;

desc USERS;

insert into USERS
(
	user_id,
    user_pw,
    user_pw_salt,
    user_name,
    user_birth,
    user_gender,
    main_oauth_channel,
    created_at,
    updated_at,
	created_by,
    updated_by
)
value
(
	'ssh9308',
    '12345678901234567890123456789012345678901234',
    '123456789012345678901234',
    '신승환',
    '19930823',
    'M',
    'kakao',
    now(),
    null,
    'system',
    null
);


select * from TELEGRAM_ROOM;

insert into TELEGRAM_ROOM 
(
	room_token,
	is_room_approved,
	created_at,
	updated_at,
	created_by,
	updated_by,
	user_seq
) value
(
	'7651339592:AAGCi-0QtWSnkh47rsmY5hkcDDwNiQMdCLs',
	1,
    now(),
    null,
    'system',
    null,
    1
);

SHOW TABLES;
SHOW INDEX FROM CONSUME_PRODT_DETAIL;

SELECT * FROM CONSUME_PRODT_DETAIL order by reg_dt desc;
SELECT * FROM CONSUME_PRODT_DETAIL ORDER BY timestamp DESC;
SELECT * FROM CONSUME_PRODT_KEYWORD;

# timestamp, prodt_name

show tables;
SELECT * FROM CONSUME_PRODT_KEYWORD WHERE consume_keyword like '%배송%';
SELECT * FROM CONSUME_PRODT_KEYWORD;
SELECT * FROM CONSUME_PRODT_DETAIL order by timestamp desc;
CREATE TABLE CONSUME_PRODT_DETAIL_DEV LIKE CONSUME_PRODT_DETAIL;
select * from CONSUME_PRODT_DETAIL order by timestamp desc;

INSERT INTO CONSUME_PRODT_DETAIL_DEV
SELECT * FROM CONSUME_PRODT_DETAIL;

show tables;



SHOW INDEX FROM CONSUME_PRODT_DETAIL;

desc CONSUME_PRODT_DETAIL;

select * from CONSUME_PRODT_DETAIL order by reg_dt desc;

select * from SPENT_DETAIL order by created_at desc;

desc SPENT_DETAIL;



SELECT CURRENT_USER();

select now();

select * from CONSUME_PRODT_KEYWORD;

select * from CONSUME_PRODT_KEYWORD;
select * from CONSUMUE_KEYWORD_TYPE;

desc COMMON_CONSUME_KEYWORD_TYPE;
insert into COMMON_CONSUME_KEYWORD_TYPE
(
	consume_keyword_type,
    created_at,
	updated_at,
	created_by,
	updated_by
)
select
	consume_keyword_type,
    now(),
    null,
    'seunghwan_dev',
    null
from CONSUMUE_KEYWORD_TYPE;

select * from COMMON_CONSUME_KEYWORD_TYPE;

select * from CONSUME_PRODT_KEYWORD;

desc COMMON_CONSUME_PRODT_KEYWORD;

select * from COMMON_CONSUME_PRODT_KEYWORD;

insert into COMMON_CONSUME_PRODT_KEYWORD
(
	consume_keyword,
	keyword_weight,
	created_at,
	updated_at,
	created_by,
	updated_by,
	consume_keyword_type_id
)select
	c1.consume_keyword,
    1,
    now(),
    null,
    'seunghwan_dev',
    null,
    c2.consume_keyword_type_id
from CONSUME_PRODT_KEYWORD c1
inner join COMMON_CONSUME_KEYWORD_TYPE c2 on c1.consume_keyword_type = c2.consume_keyword_type;

select * from CONSUME_PRODT_KEYWORD;
select * from COMMON_CONSUME_KEYWORD_TYPE;

select count(*) from CONSUME_PRODT_KEYWORD;


select * from COMMON_CONSUME_KEYWORD_TYPE;

truncate COMMON_CONSUME_KEYWORD_TYPE;


##COMMON_CONSUME_KEYWORD_TYPE
##COMMON_CONSUME_PRODT_KEYWORD
# CONSUME_PRODT_KEYWORD
# CONSUMUE_KEYWORD_TYPE

CREATE table SPENT_DETAIL (
	spent_idx bigint not null AUTO_INCREMENT comment '지출 고유번호',
	spent_name varchar(200) not null comment '지출이름',
    spent_money int not null comment '지출금',
    spent_at datetime not null default CURRENT_TIMESTAMP comment '지출 시각',
    should_index boolean not null comment '색인 대상 여부',
	created_at datetime not null default CURRENT_TIMESTAMP comment '생성 시각',
    updated_at datetime null comment '수정 시각',
    created_by varchar(100) not null default 'system' comment '생성자',
	updated_by varchar(100) null comment '수정자',
    primary key (spent_idx)
);



CREATE TABLE APP_USER (
	user_id BIGINT AUTO_INCREMENT PRIMARY KEY comment '유저 아이디',
	telegram_chat_id BIGINT NOT NULL comment 'telegram 채팅방 id',
	timezone VARCHAR(50) NOT NULL DEFAULT 'Asia/Seoul' comment '타입존',
	created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at datetime null comment '수정 시각',
	created_by varchar(100) not null default 'system' comment '생성자',
	updated_by varchar(100) null comment '수정자'
);


SELECT * FROM CONSUME_PRODT_KEYWORD_V1;

CREATE TABLE CONSUME_PRODT_KEYWORD_V1 (
	consume_keyword_type varchar(100) not null comment '소비 키워드',
    consume_keyword varchar(200) not null comment '소비 타이틀',
    created_at datetime not null default CURRENT_TIMESTAMP comment '생성 시각',
    updated_at datetime null comment '수정 시각',
    created_by varchar(100) not null default 'system' comment '생성자',
	updated_by varchar(100) null comment '수정자',
    
    primary key (consume_keyword_type, consume_keyword)
) ENGINE=InnoDB;



# timestamp

DELETE FROM CONSUME_PRODT_DETAIL
where prodt_name = '(주)버킷플레이스' and timestamp = '2026-01-06 20:05:00';

SELECT * FROM CONSUME_PRODT_DETAIL where prodt_name = 'test' and prodt_money = 1234;


SELECT * FROM CONSUME_PRODT_KEYWORD;

SELECT * FROM CONSUME_PRODT_KEYWORD where consume_keyword like '%대출%';

## 대출관련
## 집관련

SELECT * FROM CONSUMUE_KEYWORD_TYPE;
DESC CONSUMUE_KEYWORD_TYPE;
desc CONSUME_KEYWORD_TYPE_V1;

select * from CONSUME_KEYWORD_TYPE_V1;

insert into CONSUME_KEYWORD_TYPE_V1
(
	consume_keyword_type
)
select
	consume_keyword_type
from CONSUMUE_KEYWORD_TYPE;


select * from CONSUME_PRODT_KEYWORD_V1;


SELECT count(*) FROM CONSUME_PRODT_KEYWORD;
SELECT * FROM CONSUME_PRODT_KEYWORD;
DESC CONSUME_PRODT_KEYWORD;

insert into CONSUME_PRODT_KEYWORD_V1
(
	consume_keyword_type, consume_keyword
)
select
	consume_keyword_type, consume_keyword from CONSUME_PRODT_KEYWORD;

select * from CONSUME_PRODT_KEYWORD_V1;

CREATE TABLE CONSUME_PRODT_KEYWORD_V1 (
	consume_keyword_type varchar(100) not null comment '소비 키워드',
    consume_keyword varchar(200) not null comment '소비 타이틀',
    created_at datetime not null default CURRENT_TIMESTAMP comment '생성 시각',
    updated_at datetime null comment '수정 시각',
    created_by varchar(100) not null default 'system' comment '생성자',
	updated_by varchar(100) null comment '수정자',
    
    primary key (consume_keyword_type, consume_keyword)
) ENGINE=InnoDB;

SHOW INDEX FROM CONSUME_PRODT_KEYWORD;

desc CONSUME_PRODT_DETAIL;

select * from CONSUME_PRODT_DETAIL;

select * from CONSUME_KEYWORD_TYPE_V1;

select * from CONSUME_PRODT_KEYWORD_V1;


select * from CONSUME_KEYWORD_TYPE_V1;


drop table COMMON_CONSUME_KEYWORD_TYPE;

CREATE TABLE COMMON_CONSUME_KEYWORD_TYPE
(
  consume_keyword_type_id BIGINT   auto_increment    NOT NULL COMMENT '대표 키워드 타입 아이디',
  consume_keyword_type    VARCHAR(100) NOT NULL COMMENT '대표 키워드 타입',
  created_at              DATETIME     NOT NULL COMMENT '생성 시각',
  updated_at              DATETIME     NULL     COMMENT '수정 시각',
  created_by              VARCHAR(100) NOT NULL COMMENT '생성자',
  updated_by              VARCHAR(100) NULL     COMMENT '수정자',
  PRIMARY KEY (consume_keyword_type_id)
) ENGINE=InnoDB COMMENT '공통 키워드 타입 테이블';

alter table COMMON_CONSUME_KEYWORD_TYPE
add constraint uk_common_consume_keyword_type_consume_keyword_type
unique (consume_keyword_type);

desc COMMON_CONSUME_KEYWORD_TYPE;



CREATE TABLE COMMON_CONSUME_PRODT_KEYWORD
(
  consume_keyword_id      BIGINT   auto_increment    NOT NULL COMMENT '키워드 아이디',
  consume_keyword         VARCHAR(200) NOT NULL COMMENT '키워드',
  keyword_weight          INT          NOT NULL COMMENT '키워드 가중치',
  created_at              DATETIME     NOT NULL COMMENT '생성 시각',
  updated_at              DATETIME     NULL     COMMENT '수정 시각',
  created_by              VARCHAR(100) NOT NULL COMMENT '생성자',
  updated_by              VARCHAR(100) NULL     COMMENT '수정자',
  consume_keyword_type_id BIGINT       NOT NULL COMMENT '대표 키워드 타입 아이디',
  PRIMARY KEY (consume_keyword_id)
) ENGINE=InnoDB COMMENT '공통 소비-키워드 연관 테이블';

alter table COMMON_CONSUME_PRODT_KEYWORD
add constraint uk_common_consume_prodt_keyword_consume_keyword
unique (consume_keyword);

create index idx_common_consume_prodt_keyword_consume_keyword_type_id on COMMON_CONSUME_PRODT_KEYWORD (consume_keyword_type_id);

desc COMMON_CONSUME_PRODT_KEYWORD;





CREATE TABLE SPENT_GROUP_INFO
(
  spent_group_id BIGINT    auto_increment   NOT NULL COMMENT '지출 그룹 고유번호',
  spent_group_nm VARCHAR(100) NULL     COMMENT '지출 그룹 이름',
  status         VARCHAR(20)  NULL     COMMENT '상태',
  created_at     DATETIME     NOT NULL COMMENT '생성 시각',
  updated_at     DATETIME     NULL     COMMENT '수정 시각',
  created_by     VARCHAR(100) NOT NULL COMMENT '생성자',
  updated_by     VARCHAR(100) NULL     COMMENT '수정자',
  PRIMARY KEY (spent_group_id)
) ENGINE=InnoDB COMMENT '지출내역 그룹';

alter table SPENT_GROUP_INFO
add constraint uk_spent_group_info_spent_group_nm
unique (spent_group_nm);

desc SPENT_GROUP_INFO;


CREATE TABLE TELEGRAM_ROOM
(
  room_seq         BIGINT   auto_increment    NOT NULL COMMENT 'telegram 방 물리 식별자',
  room_token       VARCHAR(400) NOT NULL COMMENT 'telegram 방 논리 식별자',
  is_room_approved BOOL         NOT NULL COMMENT 'telegram 방 승인 여부',
  created_at       DATETIME     NOT NULL COMMENT '생성 시각',
  updated_at       DATETIME     NULL     COMMENT '수정 시각',
  created_by       VARCHAR(100) NOT NULL COMMENT '생성자',
  updated_by       VARCHAR(100) NULL     COMMENT '수정자',
  user_seq         BIGINT       NOT NULL COMMENT '유저 식별번호',
  PRIMARY KEY (room_seq)
) ENGINE=InnoDB COMMENT 'Telegram room 테이블';

create index idx_telegram_room_user_seq on TELEGRAM_ROOM (user_seq);


alter table TELEGRAM_ROOM
add constraint uk_telegram_room_room_token
unique (room_token);


CREATE TABLE USERS
(
  user_seq           BIGINT    auto_increment   NOT NULL COMMENT '유저 식별번호',
  user_id            VARCHAR(50)  NOT NULL COMMENT '유저 아이디',
  user_pw            CHAR(44)     NOT NULL COMMENT '유저 비밀번호',
  user_pw_salt       CHAR(24)     NOT NULL COMMENT '유저 비밀번호 솔트값',
  user_name          VARCHAR(50)  NOT NULL COMMENT '유저 이름',
  user_birth         VARCHAR(8)   NOT NULL COMMENT '유저 생년월일',
  user_gender        CHAR(1)      NULL     COMMENT '유저 성별',
  main_oauth_channel VARCHAR(25)  NULL     COMMENT '유저 oauth2 메인 채널',
  created_at         DATETIME     NOT NULL COMMENT '생성 시각',
  updated_at         DATETIME     NULL     COMMENT '수정 시각',
  created_by         VARCHAR(100) NOT NULL COMMENT '생성자',
  updated_by         VARCHAR(100) NULL     COMMENT '수정자',
  PRIMARY KEY (user_seq)
) ENGINE=InnoDB COMMENT '유저 테이블';

alter table USERS
add constraint uk_users_user_id
unique (user_id);





