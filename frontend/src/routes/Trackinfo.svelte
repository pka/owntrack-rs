<script>
    import { PUBLIC_BASE_URL } from "$env/static/public";
    import { trackQuery } from "./datetime.js";

    let { curTrack, setTrackpoints } = $props();
    let loader = $derived(load_trackpoints(curTrack));

    async function load_trackpoints(track) {
        if (!track) return null;
        const res = await fetch(
            `${PUBLIC_BASE_URL}/trackpoints?${trackQuery(track)}`,
        );
        const json = await res.json();
        setTrackpoints(json);
        return json;
    }
</script>

{#await loader then trackpoints}
    {#if trackpoints && curTrack}
        {@const track = trackpoints.stats}
        <ul>
            <li>tid: <b>{curTrack.tid}</b></li>
            <li>User: {curTrack.user_id}</li>
            <li>Device: {curTrack.device}</li>
            <li>Distance: {(track.distance / 1000).toFixed(1)} km</li>
            <li>
                Speed: {track.min_speed}-{track.max_speed} (Ø {Math.round(
                    track.mean_speed,
                )}) km/h
            </li>
            <li>
                Altitude: {track.min_elevation}-{track.max_elevation} (Ø {Math.round(
                    track.mean_elevation,
                )}) müM
            </li>
            <li>
                Elevation: ↗{track.elevation_up}m ↘{track.elevation_down}m
            </li>
        </ul>
    {/if}
{:catch error}
    <p style="color: red">{error.message}</p>
{/await}
