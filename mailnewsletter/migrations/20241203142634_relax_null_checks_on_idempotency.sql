alter table idempotency alter column response_status_code drop not null;
alter table idempotency alter column response_body drop not null;
alter table idempotency alter column response_headers drop not null;
