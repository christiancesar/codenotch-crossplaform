-- Fixture for the Cursor state.vscdb ItemTable.
-- Values are dummy/redacted; only key names and non-secret plan metadata are preserved.
CREATE TABLE IF NOT EXISTS ItemTable(key TEXT PRIMARY KEY, value TEXT);
INSERT OR REPLACE INTO ItemTable(key, value) VALUES
  ('cursorAuth/accessToken', 'test-access-token-redacted'),
  ('cursorAuth/stripeMembershipAuthId', 'test-auth-id-redacted'),
  ('cursorAuth/stripeMembershipType', 'free'),
  ('cursorAuth/cachedEmail', 'test@example.com');
