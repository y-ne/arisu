CREATE TABLE cameras (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    uri TEXT NOT NULL,
    latitude DOUBLE PRECISION NOT NULL,
    longitude DOUBLE PRECISION NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_cameras_active ON cameras (is_active) WHERE deleted_at IS NULL;
CREATE INDEX idx_cameras_created_by ON cameras (created_by);
