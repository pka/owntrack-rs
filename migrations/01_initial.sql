-- CREATE SEQUENCE devices_id_seq;
CREATE TABLE devices (
    id INTEGER PRIMARY KEY, -- DEFAULT NEXTVAL ('devices_id_seq')
    user_id VARCHAR(200) NOT NULL,
    device VARCHAR(200) NOT NULL,
    -- last position information
    lat DOUBLE PRECISION NOT NULL,
    lon DOUBLE PRECISION NOT NULL,
    ts TIMESTAMPTZ NOT NULL,
    tid VARCHAR(10) NOT NULL,
    velocity SMALLINT, -- USMALLINT
    alt SMALLINT, -- USMALLINT
    accuracy INTEGER, -- UINTEGER
    v_accuracy SMALLINT,
    cog SMALLINT -- TINYINT
);

CREATE UNIQUE INDEX user_device_idx ON devices (user_id, device);

-- CREATE SEQUENCE positions_id_seq;
CREATE TABLE positions (
    id INTEGER PRIMARY KEY, -- DEFAULT NEXTVAL ('positions_id_seq')
    device_id INTEGER NOT NULL,
    lat DOUBLE PRECISION NOT NULL,
    lon DOUBLE PRECISION NOT NULL,
    ts TIMESTAMPTZ NOT NULL,
    tid VARCHAR(10) NOT NULL,
    velocity SMALLINT, -- USMALLINT
    alt SMALLINT, -- USMALLINT
    accuracy INTEGER, -- UINTEGER
    v_accuracy SMALLINT,
    cog SMALLINT, -- TINYINT
    annotations TEXT DEFAULT '{}' NOT NULL -- TODO: JSONB for PostgreSQL
);
