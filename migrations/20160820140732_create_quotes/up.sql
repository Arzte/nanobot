create table quotes (
  id serial primary key,
  content varchar(2000) not null,
  message_id bigint not null,
  quoted_at timestamp without time zone default (now() at time zone 'utc'),
  quoted_by bigint not null,
  server_id bigint not null
)
