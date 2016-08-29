create table configs (
  id serial primary key,
  channel_id bigint,
  key varchar(100) not null,
  kind smallint not null,
  server_id bigint,
  value varchar(2000) not null
)
