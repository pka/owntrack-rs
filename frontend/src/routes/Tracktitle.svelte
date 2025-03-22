<script>
    import { utcToLocalDate, utcToLocalTime } from "./datetime.js";

    let { curTrack } = $props();
</script>

<img src="/favicon.svg" alt="favicon" width="15" height="15" />&nbsp;
{#if curTrack}
    Track from {utcToLocalDate(curTrack.ts_end)}
    {utcToLocalTime(curTrack.ts_start)} - {utcToLocalTime(curTrack.ts_end)}
{/if}
