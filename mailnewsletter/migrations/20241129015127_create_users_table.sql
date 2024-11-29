create table users(
    user_id uuid primary key,
    username text not null unique,
    password text not null
);
