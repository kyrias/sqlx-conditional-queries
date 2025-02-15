CREATE TABLE items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT NOT NULL
);

-- Insert 100 sample items
WITH RECURSIVE numbers AS (
    SELECT 1 as n
    UNION ALL
    SELECT n + 1 FROM numbers WHERE n < 100
)
INSERT INTO items (name, description)
SELECT
    'Item ' || n,
    'Description for item ' || n
FROM numbers;
