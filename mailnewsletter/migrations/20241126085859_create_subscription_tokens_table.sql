create table subscription_tokens (
    subscription_token text not null,
    subscriber_id uuid not null references subscriptions(id),
    primary key (subscription_token)
);
