-- 创建subscriptions表
create table subscriptions
(
    id uuid not null,
    primary key (id),
    email text not null unique,
    name text not null,
    subscribed_at timestamptz not null
);

comment on column subscriptions.id is '订阅ID';
comment on column subscriptions.email is '订阅用户邮箱地址';
comment on column subscriptions.name is '订阅用户姓名';
comment on column subscriptions.subscribed_at is '订阅时间';
