create table music_requests (
  id serial primary key,
  channel_id bigint not null,
  content varchar(2000) not null,
  member_id bigint not null,
  server_id bigint not null
)
