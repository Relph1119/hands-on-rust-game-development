alter table subscriptions add column status text null;
comment on column subscriptions.status is '订阅状态';
