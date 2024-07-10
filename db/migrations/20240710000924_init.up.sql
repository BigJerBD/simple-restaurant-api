-- Add up migration script here

CREATE TABLE IF NOT EXISTS restaurant_tables (
    table_number INT PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS restaurant_table_orders (
    id SERIAL PRIMARY KEY,
    table_number INT,
    item_name VARCHAR(255) NOT NULL,
    -- snapshot ? Finishes at?
    cook_time_seconds INT NOT NULL,
    CONSTRAINT fk_table_number FOREIGN KEY (table_number) REFERENCES restaurant_tables(table_number)
);

CREATE INDEX idx_table_number ON restaurant_table_orders (table_number)
