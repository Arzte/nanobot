create table members (
  id serial primary key,
  message_count bigint not null default 0,
  nickname varchar(100),
  server_id bigint not null,
  user_id bigint not null,
  weather_location varchar(150)
)
