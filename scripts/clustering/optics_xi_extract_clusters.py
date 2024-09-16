import numpy as np
import sklearn.cluster as cluster

dataset = np.load("optics_clustering_dataset.npy")

clustering = cluster.OPTICS(min_samples=8,cluster_method='dbscan',xi=0.01,eps=0.01).fit(dataset)

stops_with_cluster = list(map(lambda x: [x[0], x[1][0], x[1][1]], zip(clustering.labels_, dataset)))
np.savetxt("optics_clusters.csv", np.asarray(stops_with_cluster), header="cluster,lat,lon", fmt="%f", delimiter=",")