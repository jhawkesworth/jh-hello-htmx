CREATE TABLE if not exists todos (
                       id serial PRIMARY KEY,
                       created TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                       done BOOLEAN NOT NULL DEFAULT FALSE,
                       note TEXT NOT NULL
);