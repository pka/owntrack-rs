export function addDays(date, days) {
  const newDate = new Date(date);
  newDate.setDate(date.getDate() + days);
  return newDate;
}

// The API returns UTC timestamps like '2025-03-01 08:21:16+00'.
// Normalise them to ISO 8601 ('2025-03-01T08:21:16+00:00') so that every
// browser parses them as the same instant, instead of as local time.
export function parseUtc(utcTimeString) {
  if (utcTimeString instanceof Date) return utcTimeString;
  return new Date(
    utcTimeString.replace(" ", "T").replace(/([+-]\d{2})$/, "$1:00"),
  );
}

export function utcToLocalTime(utcTimeString) {
  return parseUtc(utcTimeString).toLocaleTimeString("de-CH");
}

export function utcToLocalDate(utcTimeString) {
  return parseUtc(utcTimeString).toLocaleDateString("de-CH");
}

// Local calendar date as 'YYYY-MM-DD', for a Date or a UTC timestamp string.
export function isoDateString(dateTime) {
  const d = parseUtc(dateTime);
  const pad = (n) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

// Offset of local time from UTC in minutes (e.g. 660 for Australia/Sydney).
// A Date means that calendar day, so the offset is taken at its local midnight.
// A timestamp string uses its own instant.
export function utcOffsetMinutes(dateTime) {
  let d = parseUtc(dateTime);
  if (dateTime instanceof Date) {
    d = new Date(d.getFullYear(), d.getMonth(), d.getDate());
  }
  return -d.getTimezoneOffset();
}

// Query string that identifies a track for the track endpoints.
export function trackQuery(track) {
  const ts = encodeURIComponent(track.ts_start);
  return `device_id=${track.device_id}&ts_start=${ts}&tz=${utcOffsetMinutes(track.ts_start)}`;
}

export function isToday(date) {
  return isoDateString(date) === isoDateString(new Date());
}
