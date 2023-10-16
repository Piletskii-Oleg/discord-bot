-- Add migration script here
DROP TABLE birthdays;

CREATE TABLE birthdays
(
    birthday_id INTEGER PRIMARY KEY,
    birth_day   INTEGER     NOT NULL,
    birth_month INTEGER     NOT NULL,
    name        VARCHAR(30) NOT NULL
)