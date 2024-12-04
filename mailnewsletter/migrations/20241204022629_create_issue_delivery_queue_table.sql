create table issue_delivery_queue(
    newsletter_issue_id uuid not null references newsletter_issues(newsletter_issue_id),
    subscriber_email text not null,
    primary key (newsletter_issue_id, subscriber_email)
);
