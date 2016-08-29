create table channels (
  id bigint primary key,
  guild_id bigint not null,
  kind smallint not null,
  name varchar(200) not null,
  topic varchar(1024),
  user_limit smallint
)
