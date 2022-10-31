local deploy() = {
    name: "deploy",
    kind: "pipeline",
    type: "docker",
    volumes: [{name: "docker", host: {path: "/var/run/docker.sock"}}],
    trigger: {branch: ["main"]},
    steps: [
        {
            name: "run-tests",
            image: "proget.makedeb.org/docker/makedeb/makedeb:ubuntu-jammy",
            volumes: [{name: "docker", path: "/var/run/docker.sock"}],
            commands: [
                "sudo chown 'makedeb:makedeb' ./ -R",
                ".drone/scripts/setup-pbmpr.sh",
                "sudo apt-get install toast -y",
                "sudo toast"
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
        }
    ]
};

[deploy()]
