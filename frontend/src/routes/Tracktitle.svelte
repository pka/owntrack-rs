<script>
    // https://svelte-ionicons.codewithshin.com/icons
    import Paw from "svelte-ionicons/Paw.svelte";
    import { utcToLocalDate } from "./datetime.js";

    let { curTrack } = $props();
</script>

<Paw size="15" />&nbsp;
{#if curTrack}
    Track from {utcToLocalDate(curTrack.ts_end)}
{/if}
