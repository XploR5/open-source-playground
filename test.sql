
-- Expected
WITH c AS (
    SELECT customer_id, email, ROW_NUMBER() OVER (ORDER BY random()) AS rn
    FROM customers
), o AS (
    SELECT order_id, ROW_NUMBER() OVER (ORDER BY order_id) AS rn
    FROM orders
    WHERE customer_id IS NULL
)
UPDATE orders
SET customer_id = c.customer_id,
    email = c.email
FROM c
JOIN o ON c.rn = ((o.rn - 1) % (SELECT COUNT(*) FROM c)) + 1
WHERE orders.order_id = o.order_id;


-- Recieved
WITH c AS (
        SELECT customers, email, ROW_NUMBER() OVER (ORDER BY random()) AS rn
        FROM customers
    ), o AS (
        SELECT order_id, ROW_NUMBER() OVER (ORDER BY order_id) AS rn
        FROM orders
        WHERE customer_id IS NULL
    )
    UPDATE orders
    SET customers = c.customers, email = c.email
    FROM c
    JOIN o ON c.rn = ((o.rn - 1) % (SELECT COUNT(*) FROM c)) + 1
    WHERE orders.order_id = o.order_id;