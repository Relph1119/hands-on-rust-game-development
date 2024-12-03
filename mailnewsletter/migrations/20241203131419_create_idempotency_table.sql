create type header_pair as
(
    name  text,
    value bytea
);

create table idempotency (
    user_id uuid not null references users(user_id),
    idempotency_key text not null,
    response_status_code smallint not null,
    response_headers header_pair[] not null,
    response_body bytea not null,
    created_at timestamp not null,
    primary key(user_id, idempotency_key)
);
