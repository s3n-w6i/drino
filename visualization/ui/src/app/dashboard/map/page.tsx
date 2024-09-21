"use client"

import * as React from "react";

import {FullscreenControl, Map, NavigationControl, ScaleControl, useControl} from 'react-map-gl/maplibre';
import {MapboxOverlay} from '@deck.gl/mapbox';
import type {PickingInfo} from '@deck.gl/core';
import {DeckProps} from '@deck.gl/core';
import {ScatterplotLayer} from '@deck.gl/layers';
import {CSVLoader} from '@loaders.gl/csv';
import {DataFilterExtension} from '@deck.gl/extensions';
import 'maplibre-gl/dist/maplibre-gl.css';

import {X} from "lucide-react";
import {Card, CardContent, CardDescription, CardHeader, CardTitle} from "@/components/ui/card";
import {Button} from "@/components/ui/button";
import {Switch} from "@/components/ui/switch";

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

const STOP_CLUSTER_COLORS = [
    [255, 0, 0], [0, 255, 0], [0, 0, 255],
    [255, 200, 0], [0, 255, 255], [255, 0, 255],
]

interface FilterControls {
    position: string,
    filters: string[]
}

export default function MapPage() {

    const [clusterId, setClusterId] = React.useState<number | null>(null);

    const layers = [
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
            data: "http://localhost:3001/data-files/tmp/stp/stops_clustered.csv",
            loaders: [CSVLoader],

            getFillColor: (s: ClusteredStop) => (
                STOP_CLUSTER_COLORS[s.cluster_id % STOP_CLUSTER_COLORS.length]
            ),
            stroked: true,
            getLineColor: [0, 0, 0, 100],
            getLineWidth: 3,
            lineWidthMaxPixels: 3,
            getPosition: (s: ClusteredStop) => ([s.lon, s.lat]),
            getRadius: 14,
            radiusMinPixels: 2,
            radiusMaxPixels: 10,

            pickable: true,
            onClick: ({object}, event) => {
                setClusterId(object.cluster_id);
            },

            getFilterValue: (s: ClusteredStop) => s.cluster_id,
            extensions: [new DataFilterExtension({filterSize: 1})],
            filterEnabled: clusterId != null,
            filterRange: clusterId != null ? [clusterId, clusterId] : []
        })
    ];


    // Callback to populate the default tooltip with content
    const getTooltip = React.useCallback(({object}: PickingInfo<ClusteredStop>) => {
        return object && {
            html: `<b>Internal Stop ID:</b> ${object.stop_id}<br/><b>Cluster:</b> ${object.cluster_id}`
        };
    }, []);

    return (
        <div className="flex items-start flex-row">
            <div className="relative flex-1">
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
                        getTooltip={getTooltip}
                        interleaved/>

                </Map>

                {(clusterId != null) && (
                    <div className="absolute top-0 left-0 px-4 py-2">
                        <Button size="sm" className="flex-row gap-1"
                               onClick={() => setClusterId(null)}>
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
