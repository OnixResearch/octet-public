#!/usr/bin/env python3
"""Consumer-visible admission boundaries for raw legacy and Nix v4 exports."""
import copy
import unittest

import export as producer

ROOT_NAME = "0" * 32 + "-owner-job.drv"
DEP_NAME = "1" * 32 + "-tool.drv"
SOURCE_NAME = "2" * 32 + "-source"
ROOT = "/nix/store/" + ROOT_NAME
SOURCE = "/nix/store/" + SOURCE_NAME


def graph(versioned):
    if versioned:
        return {
            "version": 4,
            "derivations": {
                ROOT_NAME: {
                    "version": 4,
                    "inputs": {"drvs": {}, "srcs": [SOURCE_NAME]},
                }
            },
        }
    return {ROOT: {"inputDrvs": {}, "inputSrcs": [SOURCE]}}


def root_node(value, versioned):
    return value["derivations"][ROOT_NAME] if versioned else value[ROOT]


class ProducerGraphTests(unittest.TestCase):
    def test_legacy_and_v4_preserve_raw_graph_and_resolve_exact_sources(self):
        for versioned, label in [(False, "legacy"), (True, "v4")]:
            with self.subTest(format=label):
                value = graph(versioned)
                original = copy.deepcopy(value)
                self.assertEqual(
                    producer.producer_graph(value, ROOT, SOURCE),
                    ({SOURCE}, "nix-derivation-show-" + label),
                )
                self.assertEqual(value, original)

    def test_unknown_and_mixed_versions_are_not_relabelled(self):
        cases = []
        unknown = graph(True)
        unknown["version"] = 5
        cases.append(unknown)
        mixed_v4 = graph(True)
        root_node(mixed_v4, True)["inputSrcs"] = [SOURCE]
        cases.append(mixed_v4)
        mixed_legacy = graph(False)
        root_node(mixed_legacy, False)["version"] = 4
        cases.append(mixed_legacy)
        unknown_node = graph(True)
        root_node(unknown_node, True)["version"] = 5
        cases.append(unknown_node)
        for value in cases:
            with self.subTest(graph=value):
                with self.assertRaises(ValueError):
                    producer.producer_graph(value, ROOT, SOURCE)

    def test_missing_root_and_owner_source_fail_closed(self):
        for versioned in [False, True]:
            with self.subTest(versioned=versioned):
                value = graph(versioned)
                with self.assertRaises(ValueError):
                    producer.producer_graph(value, "/nix/store/" + DEP_NAME, SOURCE)
                node = root_node(value, versioned)
                if versioned:
                    node["inputs"]["srcs"] = []
                else:
                    node["inputSrcs"] = []
                with self.assertRaises(ValueError):
                    producer.producer_graph(value, ROOT, SOURCE)

    def test_missing_recursive_dependency_is_rejected_in_both_formats(self):
        for versioned in [False, True]:
            with self.subTest(versioned=versioned):
                value = graph(versioned)
                node = root_node(value, versioned)
                if versioned:
                    node["inputs"]["drvs"][DEP_NAME] = {
                        "outputs": ["out"], "dynamicOutputs": {}
                    }
                else:
                    node["inputDrvs"]["/nix/store/" + DEP_NAME] = ["out"]
                with self.assertRaises(ValueError):
                    producer.producer_graph(value, ROOT, SOURCE)

    def test_dynamic_output_selection_is_not_an_ordinary_input(self):
        value = graph(True)
        nodes = value["derivations"]
        nodes[DEP_NAME] = {"version": 4, "inputs": {"drvs": {}, "srcs": []}}
        selection = {"outputs": ["out"], "dynamicOutputs": {}}
        nodes[ROOT_NAME]["inputs"]["drvs"][DEP_NAME] = selection
        self.assertEqual(producer.producer_graph(value, ROOT, SOURCE)[0], {SOURCE})
        selection["dynamicOutputs"] = {"out": {"outputs": ["out"]}}
        with self.assertRaises(ValueError):
            producer.producer_graph(value, ROOT, SOURCE)

    def test_non_store_and_mixed_shape_source_references_are_rejected(self):
        for versioned, invalid in [(False, "/etc/passwd"), (True, SOURCE)]:
            with self.subTest(versioned=versioned, source=invalid):
                value = graph(versioned)
                node = root_node(value, versioned)
                sources = node["inputs"]["srcs"] if versioned else node["inputSrcs"]
                sources.append(invalid)
                with self.assertRaises(ValueError):
                    producer.producer_graph(value, ROOT, SOURCE)


if __name__ == "__main__":
    unittest.main()
