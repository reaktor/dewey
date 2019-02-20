
INSERT INTO users(id, full_name, display_name)
VALUES
  (901, 'Adam 🇯🇵', '>TESTER? Adam 1'),
  (902, 'Ben 🇺🇸', '>TESTER? Ben 2'),
  (903, 'Carly 🇨🇦', '>TESTER? Carly 3');

INSERT INTO properties(id, created_by, display, property_type)
VALUES
  (920, 0, '🏰 Company', 'choice');

INSERT INTO property_value_choices(id, property_id, display, created_by)
VALUES
  (92001, 920, '👟 Adidas', 0),
  (92002, 920, '🎥 HBO', 0),
  (92003, 920, '🍿 Netflix', 0),
  (92004, 920, '👠 Zappos', 0),
  (92005, 920, '🏭 Guten', 0);

INSERT INTO objects(id, created_by)
VALUES
  ('PrA1Adidas', 901),
  ('PrA2HBO', 901),
  ('PrB1HBO', 902),
  ('PrB2Zappos', 902),
  ('PrC1Guten', 903),
  ('OuC0Us', 903),
  ('OuC1Us', 903);

-- Disclaimer: all data is completely ficticious and not
-- representative of any true events, dates, or persons. 
INSERT INTO text_values
  ("object_id", property_id, "value", created_by)
VALUES
  ('PrA1Adidas', 1, '2018-02-12 Adidas proposal v2.pdf', 901),
  ('PrA2HBO', 1, '2015-03-23 HBO Creations.key', 901),
  ('PrB1HBO', 1, '2015-03-23 HBO Creations.key', 902),
  ('PrB2Zappos', 1, 'Zappos Automate Proposal v1.key', 902),
  ('PrC1Guten', 1, 'Guten Factory View 2017.pdf', 903),
  ('OuC0Us', 1, 'OurStory-SlideAssets.sketch', 903),
  ('OuC1Us', 1, ' 2017-CompanyPhotos.zip', 903);

INSERT INTO choice_values
  ("object_id", property_id, value_id, created_by)
VALUES
  ('PrA1Adidas', 920, 92001, 901),
  ('PrA2HBO', 920, 92002, 901),
  ('PrB1HBO', 920, 92002, 902),
  ('PrB2Zappos', 920, 92004, 901),
  ('PrC1Guten', 920, 92005, 901),
  ('OuC0Us', 920, 92005, 903),
  ('OuC0Us', 920, 92002, 903),
  ('OuC0Us', 920, 92003, 903);

-- DELETE FROM choice_values
-- WHERE created_by = 901 OR
--       created_by = 902 OR
--       created_by = 903;

-- DELETE FROM text_values
-- WHERE created_by = 901 OR
--       created_by = 902 OR
--       created_by = 903;
