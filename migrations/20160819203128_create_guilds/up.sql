create table guilds (
  id bigint primary key,
  active boolean not null default true,
  name varchar(100) not null,
  owner_id bigint not null
)
