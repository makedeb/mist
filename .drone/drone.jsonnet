local deploy() = {
    name: "deploy",
    kind: "pipeline",
    type: "docker",
    trigger: {branch: ["main"]},
    steps: [
        {
            name: "run-tests",
            image: "proget.makedeb.org/docker/makedeb/makedeb:ubuntu-focal",
            commands: [".drone/scripts/run-tests.sh"]
        },

        {
            name: "create-release",
            image: "proget.makedeb.org/docker/makedeb/makedeb:ubuntu-focal",
	    environment: {
		github_api_key: {from_secret: "github_api_key"}
	    },
            commands: [".drone/scripts/create-release.sh"]
        },

        {
            name: "publish-mpr",
            image: "proget.makedeb.org/docker/makedeb/makedeb:ubuntu-focal",
	    environment: {
	        ssh_key: {from_secret: "ssh_key"}
	    },
            commands: [".drone/scripts/publish-mpr.sh"]
        }
    ]
};

[deploy()]
