"use client"

import * as React from "react";

import {FullscreenControl, Map, NavigationControl, ScaleControl, useControl} from "react-map-gl/dist/es5/exports-maplibre";
import {MapboxOverlay} from '@deck.gl/mapbox';
import {type Color, Layer, type PickingInfo, type Position} from '@deck.gl/core';
import type {DeckProps} from '@deck.gl/core';
import {ScatterplotLayer, LineLayer} from '@deck.gl/layers';
import {CSVLoader} from '@loaders.gl/csv';
import {DataFilterExtension} from '@deck.gl/extensions';
import 'maplibre-gl/dist/maplibre-gl.css';

import {X} from "lucide-react";
import {Card, CardContent, CardDescription, CardHeader, CardTitle} from "~/components/ui/card";
import {Button} from "~/components/ui/button";
import {Switch} from "~/components/ui/switch";
// @ts-expect-error: This type is not properly exported by deck.gl
import type { TooltipContent } from '@deck.gl/core/lib/tooltip';
import type {Route} from "../../.react-router/types/app/routes/+types/home";

export function meta({}: Route.MetaArgs) {
    return [
        { title: "Map" },
    ];
}

function DeckGLOverlay(props: DeckProps) {
    const overlay = useControl<MapboxOverlay>(() => new MapboxOverlay(props));
    overlay.setProps(props);
    return null;
}

type ClusteredStop = {
    stop_id: number,
    lat: number,
    lon: number,
    cluster_id: number,
};

type TransferPattern = {
    start: number,
    start_lat: number,
    start_lon: number,
    target: number,
    target_lat: number,
    target_lon: number,
};

const STOP_CLUSTER_COLORS: Color[] = [
    [255, 0, 0], [0, 255, 0], [0, 0, 255],
    [255, 200, 0], [0, 255, 255], [255, 0, 255],
]


export default function MapPage() {

    const [clusterId, setClusterId] = React.useState<number | null>(null);
    const [stopId, setStopId] = React.useState<number | null>(null);

    let layers: Layer[] = [
        /*new GeoJsonLayer<RoadProperties>({
            id: 'geojson',
            data: "https://raw.githubusercontent.com/visgl/deck.gl-data/master/examples/highway/roads.json",
            lineWidthMinPixels: 0.5,
            getLineWidth: (f: Road) => {
                return 10;
            },

            pickable: true,

            transitions: {
                getLineColor: 1000,
                getLineWidth: 1000
            }
        }),*/
        new ScatterplotLayer<ClusteredStop>({
            id: "clustered-stops",
            data: "https://localhost:3001/data-files/tmp/stp/stops_clustered.csv",
            loaders: [CSVLoader],

            getFillColor: (s: ClusteredStop): Color => (
                STOP_CLUSTER_COLORS[s.cluster_id % STOP_CLUSTER_COLORS.length]
            ),
            stroked: true,
            getLineColor: [0, 0, 0, 100],
            getLineWidth: 3,
            lineWidthMaxPixels: 3,
            getPosition: (s: ClusteredStop): Position => ([s.lon, s.lat]),
            getRadius: 14,
            radiusMinPixels: 2,
            radiusMaxPixels: 10,

            pickable: true,
            onClick: ({ object }) => {
                if (clusterId == null) {
                    setClusterId(object.cluster_id);
                } else {
                    setStopId(object.stop_id);
                }
            },

            // @ts-expect-error: getFilterValue is an extension
            getFilterValue: (s: ClusteredStop): number => s.cluster_id,
            filterEnabled: clusterId != null,
            filterRange: clusterId != null ? [clusterId, clusterId] : [],
            extensions: [new DataFilterExtension({ filterSize: 1 })],
        })
    ];

    React.useEffect(()=> {
        if (clusterId != null) {
            // Prepend the line layer (so it's on the bottom)
            layers.unshift(
                new LineLayer<TransferPattern>({
                    id: "transfer-patterns",
                    data: `https://localhost:3001/data-files/tmp/stp/clusters/${clusterId}/tp_vis.csv`,
                    loaders: [CSVLoader],

                    getSourcePosition: (d) => ([d.start_lon, d.start_lat]),
                    getTargetPosition: (d) => ([d.target_lon, d.target_lat]),

                    widthUnits: 'meters',
                    getWidth: 3,
                    widthMinPixels: 0.5,
                    widthMaxPixels: 5,

                    // @ts-expect-error: getFilterValue is an extension
                    getFilterValue: (d: TransferPattern) => d.start,
                    filterEnabled: stopId != null,
                    filterRange: stopId != null ? [stopId, stopId] : [],
                    extensions: [new DataFilterExtension({ filterSize: 1 })]
                })
            );
        } else {
            layers = layers.filter(layer => layer.id !== "transfer-patterns");
        }
    }, [clusterId]);

    /*React.useEffect(() => {
        const loadLayer = async() => {
            const clusteredStopsFile = await fetch("http://localhost:3001/data-files/tmp/stp/stops_clustered.arrow");
            const table = await tableFromIPC(clusteredStopsFile);
            console.log(table);
            const deckLayer = new GeoArrowScatterplotLayer({
                id: "scatterplot",
                data: table,
                /// Geometry column
                getPosition: new Vector(table.getChild("lat"), table.getChild("lon"))!,
                /// Column of type FixedSizeList[3] or FixedSizeList[4], with child type Uint8
                // getFillColor: table.getChild("colors")!,
            });

            layers.push(deckLayer);
        }

        loadLayer();
    }, []);*/


    // Callback to populate the default tooltip with content
    const getTooltip = React.useCallback(({object}: PickingInfo<ClusteredStop>): TooltipContent => {
        return object && {
            html: `<b>Internal Stop ID:</b> ${object.stop_id}<br/><b>Cluster:</b> ${object.cluster_id}`
        };
    }, []);

    const clearClusterFilter = () => {
        setClusterId(null);
        setStopId(null);
    };

    return (
        <div className="flex items-_start flex-row">
            <div className="relative flex-1 rounded-r-xl overflow-hidden">
                <Map
                    initialViewState={{
                        longitude: 0,
                        latitude: 0,
                        zoom: 1
                    }}
                    mapStyle="https://basemaps.cartocdn.com/gl/positron-gl-style/style.json"
                    style={{width: "100%", height: "80vh"}}>

                    <NavigationControl position="top-right"/>
                    <FullscreenControl position="top-right"/>
                    <ScaleControl/>

                    <DeckGLOverlay
                        layers={layers}
                        controller
                        getTooltip={getTooltip} />

                </Map>

                {(clusterId != null) && (
                    <div className="absolute top-0 left-0 px-4 py-2">
                        <Button size="sm" className="flex-row gap-1"
                                onClick={clearClusterFilter}>
                            Filtered by Cluster
                            <X className="h-4 w-4"/>
                        </Button>
                    </div>
                )}
            </div>
            <Card className="w-96 mx-4">
                <CardHeader className="bg-muted/50">
                    <CardTitle>Layers</CardTitle>
                    <CardDescription>Select data to display on the map</CardDescription>
                </CardHeader>
                <CardContent className="p-4">
                    <div className="hover:bg-muted/30 rounded py-3 px-4 flex flex-row items-center gap-1">
                        <div className="flex-1">
                            <p className="font-bold">Stop clusters</p>
                            <p className="text-sm text-muted-foreground">Clustering for Scalable Transfer Patterns</p>
                        </div>
                        <Switch />
                    </div>
                </CardContent>
            </Card>
        </div>
    );
}
