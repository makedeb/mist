local deploy() = {
    name: "deploy",
    kind: "pipeline",
    type: "docker",
    trigger: {branch: ["main"]},
    steps: [
        {
            name: "run-tests",
            image: "proget.makedeb.org/docker/makedeb/makedeb:ubuntu-jammy",
            commands: [
                "sudo chown 'makedeb:makedeb' ./ -R",
                ".drone/scripts/setup-pbmpr.sh",
                "sudo apt-get install cargo libssl-dev pkg-config libapt-pkg-dev -y",
                "cargo fmt --check",
                "cargo clippy -- -D warnings"
            ]
        },

        {
            name: "create-release",
            image: "proget.makedeb.org/docker/makedeb/makedeb:ubuntu-jammy",
	    environment: {
		github_api_key: {from_secret: "github_api_key"}
	    },
            commands: [".drone/scripts/create-release.sh"]
        },

        {
            name: "publish-mpr",
            image: "proget.makedeb.org/docker/makedeb/makedeb:ubuntu-jammy",
	    environment: {
	        ssh_key: {from_secret: "ssh_key"}
	    },
            commands: [".drone/scripts/publish-mpr.sh"]
        },

        {
            name: "publish-crates-io",
            image: "proget.makedeb.org/docker/makedeb/makedeb:ubuntu-jammy",
            environment: {
                CARGO_REGISTRY_TOKEN: {from_secret: "crates_api_key"}
            },
            commands: [
                ".drone/scripts/setup-pbmpr.sh",
                "sudo apt-get install cargo libssl-dev pkg-config libapt-pkg-dev -y",
		"rm makedeb/mpr-cli -rf",
                "cargo publish"
            ]
        }
    ]
};

[deploy()]
