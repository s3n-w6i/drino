"use client"

import * as React from "react";
import {useState} from "react";

import {
    FullscreenControl,
    Map,
    NavigationControl,
    ScaleControl,
    useControl
} from "react-map-gl/dist/es5/exports-maplibre";
import {MapboxOverlay} from '@deck.gl/mapbox';
import type {DeckProps} from '@deck.gl/core';
import {type Color, Layer, type PickingInfo, type Position} from '@deck.gl/core';
import {ScatterplotLayer} from '@deck.gl/layers';
import {CSVLoader} from '@loaders.gl/csv';
import {DataFilterExtension} from '@deck.gl/extensions';
import 'maplibre-gl/dist/maplibre-gl.css';

import {X} from "lucide-react";
import {Card, CardContent, CardDescription, CardHeader, CardTitle} from "~/components/ui/card";
import {Button} from "~/components/ui/button";
import {Switch} from "~/components/ui/switch";
// @ts-expect-error: This type is not properly exported by deck.gl
import type {TooltipContent} from '@deck.gl/core/lib/tooltip';
import type {Route} from "../../.react-router/types/app/routes/+types/home";
import {GeoArrowPathLayer} from "@geoarrow/deck.gl-layers";
import {Table, tableFromIPC} from "apache-arrow";

export function meta({}: Route.MetaArgs) {
    return [
        {title: "Map"},
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
    target: number,
};

const STOP_CLUSTER_COLORS: Color[] = [
    [255, 0, 0], [0, 255, 0], [0, 0, 255],
    [255, 200, 0], [0, 255, 255], [255, 0, 255],
]


export default function MapPage() {
    const [selections, setSelections] = useState<{
        clusterId: number | null,
        stopId: number | null,
    }>({
        clusterId: null,
        stopId: null,
    });

    const [layers, setLayers] = React.useState<Layer[]>([
        new ScatterplotLayer<ClusteredStop>({
            id: "clustered-stops",
            data: "http://localhost:3001/data-files/tmp/stp/stops_clustered.csv",
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
            onClick: ({object}) => {
                if (selections.clusterId === null) {
                    setSelections({
                        ...selections,
                        clusterId: object.cluster_id
                    });
                } else {
                    setSelections({
                        ...selections,
                        stopId: object.stop_id
                    });
                }
            },

            // @ts-expect-error: getFilterValue is an extension
            getFilterValue: (s: ClusteredStop): number => s.cluster_id,
            filterEnabled: selections.clusterId != null,
            filterRange: selections.clusterId != null ? [selections.clusterId, selections.clusterId] : [],
            extensions: [new DataFilterExtension({filterSize: 1})],
        }),
    ]);

    React.useEffect(() => {
        (async () => {
            const stats = await fetch("http://localhost:3001/api/v1/stats");
            if (stats.ok) {
                const {num_clusters} = await stats.json()

                let linesTable: Table | null = null;
                let tpTable: Table | null = null;
                for (let cluster = 0; cluster < num_clusters; cluster++) {
                    const linesResp = await fetch(`http://localhost:3001/data-files/tmp/stp/clusters/${cluster}/lines_geo.arrow`);
                    const newLinesTable = await tableFromIPC(linesResp);
                    if (linesTable === null) linesTable = newLinesTable
                    else linesTable = linesTable.concat(newLinesTable)

                    const tpResp = await fetch(`http://localhost:3001/data-files/tmp/stp/clusters/${cluster}/transfer_patterns.arrow`);
                    const newTpTable = await tableFromIPC(tpResp);
                    if (tpTable === null) tpTable = newTpTable
                    else tpTable = tpTable.concat(newTpTable)
                }

                // type hints
                linesTable = linesTable as Table
                tpTable = tpTable as Table

                setLayers([
                    /*new GeoArrowPathLayer({
                        id: "lines-clusters",
                        data: linesTable,
                        widthMinPixels: 2,
                        widthMaxPixels: 6,
                        getPosition: linesTable.getChild("")!,
                        getColor: [0, 0, 255],
                        opacity: 0.1,
                        pickable: true,
                        autoHighlight: true,
                        highlightColor: [0, 0, 0, 200]
                    }),*/
                    new GeoArrowPathLayer({
                        id: "transfer-patterns-clusters",
                        data: tpTable,
                        widthMinPixels: 1,
                        widthMaxPixels: 4,
                        getPosition: tpTable.getChild("")!,
                        getColor: [255, 0, 0],
                        opacity: 0.1,
                        pickable: true,
                        autoHighlight: true,
                        highlightColor: [0, 0, 0, 200],

                        extensions: [new DataFilterExtension({filterSize: 1})],
                        getFilterValue: (_: any, {index, data, target}: {
                            index: number;
                            data: Table;
                            target: any
                        }) => {
                            const recordBatch = data.data;
                            const row = recordBatch.get(index)!;
                            return row["start"]
                        },
                        filterRange: [1422, 1422],
                        filterEnabled: true
                    }),
                    ...layers
                ]);
            } else {
                console.error(stats)
            }
        })();
    }, []);

    /*React.useEffect(() => {
        (async () => {
            const resp = await fetch(`http://localhost:3001/data-files/tmp/global/lines.arrow`);
            const table = await tableFromIPC(resp);

            setLayers([
                new GeoArrowPathLayer({
                    id: "lines-global",
                    data: table,
                    widthMinPixels: 1.5,
                    widthMaxPixels: 8,
                    getPosition: table.getChild("")!,
                    getColor: [0, 255, 0],
                    opacity: 0.05,
                    pickable: true,
                    autoHighlight: true,
                    highlightColor: [0, 0, 0, 200]
                }),
                ...layers
            ]);
        })();
    }, [])*/

    /*React.useEffect(() => {
        if (selections.clusterId != null) {
            // Prepend the line layer (so it's on the bottom)
            setLayers([
                new LineLayer<TransferPattern>({
                    id: "transfer-patterns-cluster",
                    data: `http://localhost:3001/data-files/tmp/stp/clusters/${selections.clusterId}/tp_vis.csv`,
                    loaders: [CSVLoader],

                    getSourcePosition: (d) => ([d.start_lon, d.start_lat]),
                    getTargetPosition: (d) => ([d.target_lon, d.target_lat]),

                    widthUnits: 'meters',
                    getWidth: 3,
                    widthMinPixels: 0.5,
                    widthMaxPixels: 5,

                    // @ts-expect-error: getFilterValue is an extension
                    getFilterValue: (d: TransferPattern) => d.start,
                    filterEnabled: selections.stopId != null,
                    filterRange: selections.stopId != null ? [selections.stopId, selections.stopId] : [],
                    extensions: [new DataFilterExtension({filterSize: 1})]
                }),
                ...layers
            ]);
        } else {
            setLayers(layers.filter(layer => layer.id !== "transfer-patterns-cluster"));
        }
    }, [selections]);*/

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
    const getTooltip = React.useCallback(({object}: PickingInfo<ClusteredStop | TransferPattern>): TooltipContent => {
        if (object.hasOwnProperty("stop_id")) {
            return object && {
                html: `<b>Internal Stop ID:</b> ${object.stop_id}<br/><b>Cluster:</b> ${object.cluster_id}`
            };
        } else if (object instanceof TransferPattern) {

        } else {
            console.error("Unknown object type: ", object);
        }
    }, []);

    const clearClusterFilter = () => {
        setSelections({
            clusterId: null, stopId: null,
        });
    };

    const clearStopFilter = () => {
        setSelections({
            ...selections, stopId: null,
        });
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
                        getTooltip={getTooltip}/>

                </Map>

                <div className="absolute top-0 left-0 px-4 py-2 flex-row gap-2">
                    {(selections.clusterId != null) && (
                        <Button size="sm" className="flex-row gap-1" onClick={clearClusterFilter}>
                            Filtered by Cluster = {selections.clusterId}
                            <X className="h-4 w-4"/>
                        </Button>
                    )}
                    {(selections.stopId != null) && (
                        <Button size="sm" className="flex-row gap-1" variant="secondary" onClick={clearStopFilter}>
                            Filtered by Stop = {selections.stopId}
                            <X className="h-4 w-4"/>
                        </Button>
                    )}
                </div>
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
                        <Switch/>
                    </div>
                </CardContent>
            </Card>
        </div>
    );
}
