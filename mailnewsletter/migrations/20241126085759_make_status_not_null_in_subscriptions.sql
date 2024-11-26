begin;
    update subscriptions set status = 'confirmed' where status is null;
    alter table subscriptions alter column status set not null;
commit;