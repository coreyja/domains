-- Add migration script here
CREATE TABLE
  Users (
    user_id UUID PRIMARY KEY NOT NULL,
    coreyja_user_id UUID NOT NULL,
    is_active_sponsor BOOLEAN NOT NULL,
    is_admin BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
  );

CREATE UNIQUE INDEX idx_Users_coreyja_user_id ON Users (coreyja_user_id);
