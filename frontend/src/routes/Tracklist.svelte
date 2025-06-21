<script>
    import { PUBLIC_BASE_URL } from "$env/static/public";
    import {
        addDays,
        isoDateString,
        isToday,
        utcToLocalTime,
    } from "./datetime.js";

    let { date, curTrack, setDate, setCurTrack } = $props();
    let datestr = $derived(isoDateString(date));
    let loader = $derived(load_infos());

    async function load_infos() {
        const res = await fetch(
            `${PUBLIC_BASE_URL}/trackinfos?date=${datestr}`,
        );
        const json = await res.json();
        // Select first track, if there is only one for the first day
        if (
            json.length == 1 ||
            (json.length > 1 &&
                isoDateString(json[0].ts_end) !== isoDateString(json[1].ts_end))
        ) {
            setCurTrack(json[0]);
        } else {
            setCurTrack(null);
        }
        return json;
    }

    function checkSelected(track) {
        return (
            curTrack &&
            curTrack.device_id === track.device_id &&
            isoDateString(curTrack.ts_start) === isoDateString(track.ts_start)
        );
    }

    let track_dropdown = $state(null);

    function toggleDropdown(device_id, event) {
        event.stopPropagation();
        if (track_dropdown === device_id) {
            track_dropdown = null;
        } else {
            track_dropdown = device_id;
            // Position dropdown relative to button
            const button = event.target;
            const rect = button.getBoundingClientRect();
            const dropdown =
                button.parentElement.querySelector(".dropdown-menu");
            if (dropdown) {
                dropdown.style.top = `${rect.bottom + 2}px`;
                dropdown.style.left = `${rect.right}px`;
            }
        }
    }

    function closeDropdown() {
        track_dropdown = null;
    }
</script>

<div class="header">
    <div class="title">Owntrack-rs</div>
    <div class="date-selector">
        <button onclick={() => setDate(addDays(date, -1))}> &lt; </button>
        {datestr}
        <button onclick={() => setDate(addDays(date, 1))}> &gt; </button>
    </div>
</div>

<div class="tracks-container">
    {#await loader}
        <p>loading track list...</p>
    {:then tracks}
        <table class="tracks-table">
            <thead>
                <tr>
                    <th>User</th>
                    <th>Device</th>
                    <th>Start Time</th>
                    <th>End Time</th>
                    <th>Actions</th>
                </tr>
            </thead>
            <tbody>
                {#each tracks as track}
                    <tr
                        class={checkSelected(track) ? "selected" : ""}
                        onclick={() => setCurTrack(track)}
                    >
                        <td>{track.user_id}</td>
                        <td>{track.device}</td>
                        <td>{utcToLocalTime(track.ts_start)}</td>
                        <td>{utcToLocalTime(track.ts_end)}</td>
                        <td class="dropdown-cell">
                            <button
                                class="dropdown-button"
                                onclick={(e) =>
                                    toggleDropdown(track.device_id, e)}
                                >☰
                            </button>
                            {#if track_dropdown === track.device_id}
                                <div class="dropdown-menu">
                                    <a
                                        href="{PUBLIC_BASE_URL}/gpxtrack?device_id={track.device_id}&ts_start={encodeURIComponent(
                                            track.ts_start,
                                        )}"
                                        class="dropdown-item"
                                        onclick={(e) => {
                                            e.stopPropagation();
                                            closeDropdown();
                                        }}
                                    >
                                        Download GPX
                                    </a>
                                    <a
                                        href="{PUBLIC_BASE_URL}/track?device_id={track.device_id}&ts_start={encodeURIComponent(
                                            track.ts_start,
                                        )}"
                                        class="dropdown-item"
                                        onclick={(e) => {
                                            e.stopPropagation();
                                            closeDropdown();
                                        }}
                                    >
                                        Download GeoJSON
                                    </a>
                                    <a
                                        href="{PUBLIC_BASE_URL}/csvtrack?device_id={track.device_id}&ts_start={encodeURIComponent(
                                            track.ts_start,
                                        )}"
                                        class="dropdown-item"
                                        onclick={(e) => {
                                            e.stopPropagation();
                                            closeDropdown();
                                        }}
                                    >
                                        Download CSV
                                    </a>
                                </div>
                            {/if}
                        </td>
                    </tr>
                {/each}
            </tbody>
        </table>
    {:catch error}
        <p style="color: red">{error.message}</p>
    {/await}
</div>

<style>
    .header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 10px;
    }

    .title {
        font-weight: bold;
        font-size: 1.2em;
    }

    .date-selector {
        display: flex;
        align-items: center;
        gap: 5px;
    }

    .tracks-table {
        width: 100%;
        border-collapse: collapse;
        overflow-x: auto;
        overflow-y: visible;
    }

    .tracks-table th {
        text-align: left;
        padding: 8px;
        /* background-color: #f2f2f2; */
        border-bottom: 1px solid #ddd;
        font-weight: bold;
    }

    .tracks-table td {
        padding: 8px;
        border-bottom: 1px solid #eee;
    }

    .tracks-table tr {
        transition: background-color 0.2s;
    }

    .tracks-table tr:hover {
        background-color: #f0f0f0;
        cursor: pointer;
    }

    .tracks-table tr:focus {
        background-color: #f0f0f0;
    }

    .tracks-table tr.selected {
        background-color: lightblue;
    }

    .tracks-table tr.selected:hover {
        background-color: #a8d4e6;
    }

    .tracks-table tr.selected:focus {
        background-color: #a8d4e6;
    }

    .dropdown-cell {
        position: relative;
        overflow: visible;
    }

    .dropdown-button {
        background: none;
        border: none;
        font-size: 1.2em;
        cursor: pointer;
        padding: 4px 8px;
        border-radius: 4px;
        color: #666;
        transition: background-color 0.2s;
    }

    .dropdown-button:hover {
        background-color: #e9ecef;
    }

    .dropdown-menu {
        position: fixed;
        background: white;
        border: 1px solid #ddd;
        border-radius: 4px;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
        z-index: 9999;
        min-width: 150px;
        transform: translate(-100%, 0);
    }

    .dropdown-item {
        display: block;
        padding: 8px 12px;
        color: #333;
        text-decoration: none;
        border-bottom: 1px solid #eee;
        transition: background-color 0.2s;
    }

    .dropdown-item:last-child {
        border-bottom: none;
    }

    .dropdown-item:hover {
        background-color: #f8f9fa;
    }
</style>
