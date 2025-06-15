<script>
    // https://svelte-maplibre-gl.mierune.dev/
    import {
        MapLibre,
        NavigationControl,
        ScaleControl,
        GeoJSONSource,
        FeatureState,
        LineLayer,
        CircleLayer,
        SymbolLayer,
        Popup,
        Image,
    } from "svelte-maplibre-gl";
    import maplibregl from "maplibre-gl";
    const { LngLatBounds } = maplibregl;
    import "maplibre-gl/dist/maplibre-gl.css";
    import { PUBLIC_BASE_URL } from "$env/static/public";

    let map = $state.raw();
    let positions_source = $state.raw();
    let hoveredPositionFeat = $state.raw();
    let hoveredPointFeat = $state.raw();
    // cursor location
    let lnglat = $state.raw(new maplibregl.LngLat(0, 0));
    let { curTrack, trackpoints, positionsSelector, setCurTrack } = $props();

    // Zoom to trackpoints
    $effect(() => {
        if (trackpoints && map) {
            let bounds = new LngLatBounds(trackpoints.bbox);
            map.fitBounds(bounds, {
                padding: 40,
            });
        }
    });

    // Zoom to positions
    $effect(() => {
        if (positions_source && map) {
            positions_source.getData().then((data) => {
                let bounds = new LngLatBounds(data.bbox);
                map.fitBounds(bounds, {
                    padding: 40,
                });
            });
        }
    });

    function postitionToTrack(pos) {
        return { ...pos, ts_start: pos.time, ts_end: pos.time };
    }
    function objToHtml(obj) {
        var str = "";
        for (var p in obj) {
            if (Object.prototype.hasOwnProperty.call(obj, p)) {
                str += p + ": " + obj[p] + "<br/>";
            }
        }
        return str;
    }

    const pulseSize = 170;

    // Implement `StyleImageInterface` to draw a pulsing dot icon.
    const pulsingDot = {
        width: pulseSize,
        height: pulseSize,
        data: new Uint8Array(pulseSize * pulseSize * 4),

        onAdd: function () {
            const canvas = document.createElement("canvas");
            canvas.width = this.width;
            canvas.height = this.height;
            this.context = canvas.getContext("2d");
        },

        render: function () {
            const duration = 1000;
            const t = (performance.now() % duration) / duration;

            const radius = (pulseSize / 2) * 0.3;
            const outerRadius = (pulseSize / 2) * 0.7 * t + radius;
            const context = this.context;

            // Draw the outer circle.
            context.clearRect(0, 0, this.width, this.height);
            context.beginPath();
            context.arc(
                this.width / 2,
                this.height / 2,
                outerRadius,
                0,
                Math.PI * 2,
            );
            context.fillStyle = `rgba(255,239,0, ${1 - t})`;
            context.fill();

            // Update this image's data with data from the canvas.
            this.data = context.getImageData(
                0,
                0,
                this.width,
                this.height,
            ).data;

            // map.triggerRepaint();

            // image was updated
            return true;
        },
    };
</script>

<MapLibre
    class="map"
    bind:map
    autoloadGlobalCss={false}
    style="https://basemaps.cartocdn.com/gl/voyager-gl-style/style.json"
    maxZoom={17}
>
    <NavigationControl />
    <ScaleControl />
    <Image id="pulsing-dot" image={pulsingDot} options={{ pixelRatio: 2 }} />
    {#if trackpoints}
        <GeoJSONSource data={trackpoints}>
            <CircleLayer
                paint={{
                    "circle-color": "#ff0000",
                    "circle-radius": 5,
                }}
                onmousemove={(ev) => {
                    hoveredPointFeat = ev.features[0];
                    console.log(hoveredPointFeat.properties);
                    lnglat = ev.lngLat;
                    console.log(lnglat);
                }}
                onmouseout={() => {
                    hoveredPointFeat = undefined;
                }}
                minzoom={11}
            />
            {#if hoveredPointFeat}
                <FeatureState
                    id={hoveredPointFeat.id}
                    state={{ hover: true }}
                />
                <Popup {lnglat} closeButton={false}>
                    {@html objToHtml(hoveredPointFeat.properties)}
                </Popup>
            {/if}
        </GeoJSONSource>
    {/if}
    {#if curTrack}
        <GeoJSONSource
            data={`${PUBLIC_BASE_URL}/track?device_id=${curTrack.device_id}&ts_start=${curTrack.ts_start}`}
        >
            <LineLayer
                paint={{
                    "line-color": "#ff0000",
                    "line-width": 4,
                }}
            />
        </GeoJSONSource>
    {/if}
    {#if positionsSelector}
        <GeoJSONSource
            data={`${PUBLIC_BASE_URL}/positions?${positionsSelector}`}
            bind:source={positions_source}
        >
            <SymbolLayer
                layout={{
                    "icon-image": [
                        "case",
                        [">", ["get", "speed"], 0],
                        "pulsing-dot",
                        "",
                    ],
                    "icon-allow-overlap": true,
                }}
            />
            <CircleLayer
                paint={{
                    "circle-color": [
                        "case",
                        ["boolean", ["feature-state", "hover"], false],
                        "lightblue",
                        "#0000ff",
                    ],
                    "circle-radius": 20,
                }}
                onmousemove={(ev) => {
                    hoveredPositionFeat = ev.features[0];
                }}
                onmouseout={() => {
                    hoveredPositionFeat = undefined;
                }}
                onclick={(ev) => {
                    setCurTrack(postitionToTrack(ev.features[0].properties));
                }}
            />
            <SymbolLayer
                layout={{
                    "text-field": ["get", "tid"],
                    "text-size": 15,
                }}
                paint={{
                    "text-color": "#ffffff",
                }}
            />
            {#if hoveredPositionFeat}
                <!-- Set the hover state on the source for the hovered feature -->
                <FeatureState
                    id={hoveredPositionFeat.id}
                    state={{ hover: true }}
                />
            {/if}
        </GeoJSONSource>
    {/if}
</MapLibre>
