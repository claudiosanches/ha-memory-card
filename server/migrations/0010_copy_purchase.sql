ALTER TABLE copies ADD COLUMN edition TEXT;
-- YYYY-MM-DD.
ALTER TABLE copies ADD COLUMN purchased_on TEXT;
-- In the currency's minor unit, such as cents. Paid for owned copies,
-- expected for wanted ones.
ALTER TABLE copies ADD COLUMN price INTEGER;
-- ISO 4217 code, set together with price.
ALTER TABLE copies ADD COLUMN currency TEXT;
