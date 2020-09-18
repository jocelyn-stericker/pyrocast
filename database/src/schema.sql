create table if not exists meta(
  id text primary key not null,
  value text not null);

insert or replace into meta values("version", 1);

create table subscription(
  pk text primary key not null);
