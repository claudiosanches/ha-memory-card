-- UPC or EAN printed on a copy's box, digits only.
ALTER TABLE copies ADD COLUMN barcode TEXT;

CREATE INDEX copies_barcode ON copies (barcode);
