create table tags (
  id serial primary key,
  created_at bigint not null,
  key varchar(100) not null,
  owner_id bigint not null,
  server_id bigint not null,
  uses integer not null default 0,
  value varchar(2000) not null
)
