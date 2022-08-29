use std::collections::HashMap;

use petgraph::graph::NodeIndex;
use petgraph::prelude::DiGraph;
use petgraph::visit::IntoNodeReferences;
use petgraph::Direction;
use semver::Version;
use semver::VersionReq;
use unreal_modmetadata::Dependency;
use unreal_modmetadata::DownloadInfo;

use crate::error::ModLoaderWarning;

#[derive(Debug, Clone, PartialEq, Eq)]
struct GraphMod {
    mod_id: String,
    versions: Vec<Version>,
    downloads: Vec<DownloadInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModWithDependencies {
    pub mod_id: String,
    pub versions: Vec<Version>,
    pub dependencies: HashMap<String, Dependency>,
}

impl ModWithDependencies {
    pub fn new(
        mod_id: String,
        versions: Vec<Version>,
        dependencies: HashMap<String, Dependency>,
    ) -> Self {
        ModWithDependencies {
            mod_id,
            versions,
            dependencies,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DependencyGraph {
    graph: DiGraph<GraphMod, VersionReq, u32>,
    node_lookup: HashMap<String, NodeIndex>,
}

// https://github.com/dtolnay/semver/issues/170#issuecomment-734284639
fn intersect_requirements(reqs: impl IntoIterator<Item = VersionReq>) -> VersionReq {
    let reqs: Vec<_> = reqs
        .into_iter()
        .filter_map(|req| {
            if req == VersionReq::STAR {
                None
            } else {
                Some(req.to_string())
            }
        })
        .collect();

    if reqs.is_empty() {
        VersionReq::STAR
    } else {
        reqs.join(", ").parse().unwrap()
    }
}

impl DependencyGraph {
    fn get_or_add_mod(&mut self, game_mod: GraphMod) -> NodeIndex {
        if let Some(node) = self.node_lookup.get(&game_mod.mod_id) {
            let weight = self.graph.node_weight_mut(*node).unwrap();
            if weight.versions.is_empty() && !game_mod.versions.is_empty() {
                weight.versions = game_mod.versions.clone();
            }
            for download in game_mod.downloads {
                if !weight.downloads.contains(&download) {
                    weight.downloads.push(download);
                }
            }
            *node
        } else {
            let mod_id = game_mod.mod_id.clone();
            let node = self.graph.add_node(game_mod);
            self.node_lookup.insert(mod_id, node);
            node
        }
    }

    pub fn add_mods(
        &mut self,
        mods: &[ModWithDependencies],
    ) -> HashMap<String, (VersionReq, Vec<DownloadInfo>)> {
        let mut dependency_nodes = Vec::new();

        for game_mod in mods {
            let mod_node = self.get_or_add_mod(GraphMod {
                mod_id: game_mod.mod_id.clone(),
                versions: game_mod.versions.clone(),
                downloads: Vec::new(),
            });

            for (dependency_mod_id, dependency) in &game_mod.dependencies {
                let dependency_node = self.get_or_add_mod(GraphMod {
                    mod_id: dependency_mod_id.clone(),
                    versions: game_mod.versions.clone(),
                    downloads: match dependency.download.as_ref() {
                        Some(e) => Vec::from([e.clone()]),
                        None => Vec::new(),
                    },
                });

                self.graph
                    .add_edge(mod_node, dependency_node, dependency.version.clone());
                dependency_nodes.push(dependency_node);
            }
        }

        let mut version_requirements = HashMap::new();
        for dependency_node in dependency_nodes {
            let requirements = self
                .graph
                .edges_directed(dependency_node, Direction::Incoming)
                .map(|e| e.weight().clone());

            let weight = self.graph.node_weight(dependency_node).unwrap();
            let requirement = intersect_requirements(requirements);
            version_requirements.insert(
                weight.mod_id.clone(),
                (requirement, weight.downloads.clone()),
            );
        }

        version_requirements
    }

    pub fn validate_graph(&self) -> (HashMap<String, Version>, Vec<ModLoaderWarning>) {
        let mut matching_versions = HashMap::new();
        let mut warnings = Vec::new();

        for node in self.graph.node_indices() {
            let requirements = self
                .graph
                .edges_directed(node, Direction::Incoming)
                .map(|e| e.weight().clone());
            let requirement = intersect_requirements(requirements);

            let weight = self.graph.node_weight(node).unwrap();
            let matching_version = weight.versions.iter().find(|e| requirement.matches(e));

            match matching_version {
                Some(matching_version) => {
                    matching_versions.insert(weight.mod_id.clone(), matching_version.clone());
                }
                None => {
                    let warning = ModLoaderWarning::unresolved_dependency(
                        weight.mod_id.clone(),
                        self.graph
                            .neighbors_directed(node, Direction::Incoming)
                            .map(|e| {
                                (
                                    self.graph.node_weight(e).unwrap().mod_id.clone(),
                                    self.graph
                                        .edges_connecting(node, e)
                                        .next()
                                        .unwrap()
                                        .weight(),
                                )
                            })
                            .map(|(mod_id, version_req)| (mod_id, version_req.to_string()))
                            .collect::<Vec<_>>(),
                    );
                    warnings.push(warning);
                }
            }
        }

        (matching_versions, warnings)
    }

    pub fn find_mod_dependents(&self, mod_id: &str) -> Vec<String> {
        match self
            .graph
            .node_references()
            .find(|(_, graph_mod)| graph_mod.mod_id == mod_id)
        {
            Some((identifier, _)) => self
                .graph
                .neighbors_directed(identifier, Direction::Incoming)
                .map(|e| self.graph.node_weight(e).unwrap().mod_id.clone())
                .collect::<Vec<_>>(),
            None => Vec::new(),
        }
    }

    pub fn find_mod_dependents_with_version(&self, mod_id: &str) -> Vec<(String, String)> {
        let node = self
            .graph
            .node_references()
            .find(|(_, graph_mod)| graph_mod.mod_id == mod_id);

        if let Some((node, _)) = node {
            return self
                .graph
                .neighbors_directed(node, Direction::Incoming)
                .map(|e| {
                    (
                        self.graph.node_weight(e).unwrap().mod_id.clone(),
                        self.graph.find_edge_undirected(node, e),
                    )
                })
                .filter(|(_, edge)| edge.is_some())
                .map(|(mod_id, edge)| {
                    (
                        mod_id,
                        self.graph.edge_weight(edge.unwrap().0).unwrap().to_string(),
                    )
                })
                .collect::<Vec<_>>();
        }

        Vec::new()
    }
}
