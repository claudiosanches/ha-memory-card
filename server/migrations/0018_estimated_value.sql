-- What an owned copy would sell for now, in the minor unit of its own
-- currency, which can differ from the price's.
ALTER TABLE copies ADD COLUMN estimated_value INTEGER;
-- ISO 4217 code, set together with estimated_value.
ALTER TABLE copies ADD COLUMN estimated_currency TEXT;
