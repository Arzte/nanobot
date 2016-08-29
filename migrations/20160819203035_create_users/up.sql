create table users (
  id bigint primary key,
  bot boolean not null,
  discriminator smallint not null,
  username varchar(100) not null
)
