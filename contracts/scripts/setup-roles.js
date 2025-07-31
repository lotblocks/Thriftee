const { ethers } = require("hardhat");
const { getNetworkConfig } = require("../config/networks");

async function setupRoles(contractAddress, networkName, roleConfig) {
    console.log(`\n=== Setting up Roles for Contract on ${networkName} ===`);
    console.log(`Contract Address: ${contractAddress}`);
    
    try {
        // Get contract instance
        const RaffleContract = await ethers.getContractFactory("RaffleContract");
        const contract = RaffleContract.attach(contractAddress);
        
        // Get signers
        const [deployer] = await ethers.getSigners();
        console.log(`Deployer address: ${deployer.address}`);
        
        // Get role constants
        const DEFAULT_ADMIN_ROLE = await contract.DEFAULT_ADMIN_ROLE();
        const RAFFLE_MANAGER_ROLE = await contract.RAFFLE_MANAGER_ROLE();
        const OPERATOR_ROLE = await contract.OPERATOR_ROLE();
        const PAUSER_ROLE = await contract.PAUSER_ROLE();
        
        console.log("\n--- Role Constants ---");
        console.log(`DEFAULT_ADMIN_ROLE: ${DEFAULT_ADMIN_ROLE}`);
        console.log(`RAFFLE_MANAGER_ROLE: ${RAFFLE_MANAGER_ROLE}`);
        console.log(`OPERATOR_ROLE: ${OPERATOR_ROLE}`);
        console.log(`PAUSER_ROLE: ${PAUSER_ROLE}`);
        
        // Check current roles
        console.log("\n--- Current Role Assignments ---");
        const deployerHasAdmin = await contract.hasRole(DEFAULT_ADMIN_ROLE, deployer.address);
        const deployerHasManager = await contract.hasRole(RAFFLE_MANAGER_ROLE, deployer.address);
        const deployerHasOperator = await contract.hasRole(OPERATOR_ROLE, deployer.address);
        const deployerHasPauser = await contract.hasRole(PAUSER_ROLE, deployer.address);
        
        console.log(`Deployer has DEFAULT_ADMIN_ROLE: ${deployerHasAdmin}`);
        console.log(`Deployer has RAFFLE_MANAGER_ROLE: ${deployerHasManager}`);
        console.log(`Deployer has OPERATOR_ROLE: ${deployerHasOperator}`);
        console.log(`Deployer has PAUSER_ROLE: ${deployerHasPauser}`);
        
        // Setup additional roles based on configuration
        if (roleConfig && roleConfig.additionalRoles) {
            console.log("\n--- Setting up Additional Roles ---");
            
            for (const roleAssignment of roleConfig.additionalRoles) {
                const { role, addresses } = roleAssignment;
                let roleBytes32;
                
                // Convert role name to bytes32
                switch (role) {
                    case "RAFFLE_MANAGER":
                        roleBytes32 = RAFFLE_MANAGER_ROLE;
                        break;
                    case "OPERATOR":
                        roleBytes32 = OPERATOR_ROLE;
                        break;
                    case "PAUSER":
                        roleBytes32 = PAUSER_ROLE;
                        break;
                    case "DEFAULT_ADMIN":
                        roleBytes32 = DEFAULT_ADMIN_ROLE;
                        break;
                    default:
                        console.log(`❌ Unknown role: ${role}`);
                        continue;
                }\n                \n                for (const address of addresses) {\n                    try {\n                        // Check if address already has the role\n                        const hasRole = await contract.hasRole(roleBytes32, address);\n                        \n                        if (hasRole) {\n                            console.log(`✅ ${address} already has ${role}`);\n                        } else {\n                            console.log(`Granting ${role} to ${address}...`);\n                            const tx = await contract.grantRole(roleBytes32, address);\n                            await tx.wait();\n                            console.log(`✅ Granted ${role} to ${address}`);\n                        }\n                    } catch (error) {\n                        console.log(`❌ Failed to grant ${role} to ${address}: ${error.message}`);\n                    }\n                }\n            }\n        }\n        \n        // Setup emergency contacts\n        if (roleConfig && roleConfig.emergencyContacts) {\n            console.log(\"\\n--- Setting up Emergency Contacts ---\");\n            \n            for (const contact of roleConfig.emergencyContacts) {\n                try {\n                    const hasRole = await contract.hasRole(PAUSER_ROLE, contact);\n                    \n                    if (hasRole) {\n                        console.log(`✅ ${contact} already has PAUSER role`);\n                    } else {\n                        console.log(`Granting PAUSER role to emergency contact ${contact}...`);\n                        const tx = await contract.grantRole(PAUSER_ROLE, contact);\n                        await tx.wait();\n                        console.log(`✅ Granted PAUSER role to ${contact}`);\n                    }\n                } catch (error) {\n                    console.log(`❌ Failed to grant PAUSER role to ${contact}: ${error.message}`);\n                }\n            }\n        }\n        \n        // Setup operator addresses\n        if (roleConfig && roleConfig.operators) {\n            console.log(\"\\n--- Setting up Operator Addresses ---\");\n            \n            for (const operator of roleConfig.operators) {\n                try {\n                    const hasRole = await contract.hasRole(OPERATOR_ROLE, operator);\n                    \n                    if (hasRole) {\n                        console.log(`✅ ${operator} already has OPERATOR role`);\n                    } else {\n                        console.log(`Granting OPERATOR role to ${operator}...`);\n                        const tx = await contract.grantRole(OPERATOR_ROLE, operator);\n                        await tx.wait();\n                        console.log(`✅ Granted OPERATOR role to ${operator}`);\n                    }\n                } catch (error) {\n                    console.log(`❌ Failed to grant OPERATOR role to ${operator}: ${error.message}`);\n                }\n            }\n        }\n        \n        // Final role verification\n        console.log(\"\\n--- Final Role Verification ---\");\n        \n        // Get all role members (this is a simplified check)\n        const roles = [\n            { name: \"DEFAULT_ADMIN\", bytes32: DEFAULT_ADMIN_ROLE },\n            { name: \"RAFFLE_MANAGER\", bytes32: RAFFLE_MANAGER_ROLE },\n            { name: \"OPERATOR\", bytes32: OPERATOR_ROLE },\n            { name: \"PAUSER\", bytes32: PAUSER_ROLE }\n        ];\n        \n        for (const role of roles) {\n            try {\n                // We can't easily enumerate all role members, so we'll just check key addresses\n                const addressesToCheck = [deployer.address];\n                \n                if (roleConfig) {\n                    if (roleConfig.emergencyContacts) addressesToCheck.push(...roleConfig.emergencyContacts);\n                    if (roleConfig.operators) addressesToCheck.push(...roleConfig.operators);\n                    if (roleConfig.additionalRoles) {\n                        roleConfig.additionalRoles.forEach(ra => {\n                            if (ra.role === role.name.replace(\"_\", \"\")) {\n                                addressesToCheck.push(...ra.addresses);\n                            }\n                        });\n                    }\n                }\n                \n                const uniqueAddresses = [...new Set(addressesToCheck)];\n                const membersWithRole = [];\n                \n                for (const addr of uniqueAddresses) {\n                    const hasRole = await contract.hasRole(role.bytes32, addr);\n                    if (hasRole) {\n                        membersWithRole.push(addr);\n                    }\n                }\n                \n                console.log(`${role.name}: ${membersWithRole.length} members`);\n                membersWithRole.forEach(addr => console.log(`  - ${addr}`));\n                \n            } catch (error) {\n                console.log(`❌ Failed to verify ${role.name}: ${error.message}`);\n            }\n        }\n        \n        console.log(\"\\n=== Role Setup Complete ===\\n\");\n        \n        return {\n            success: true,\n            contractAddress,\n            networkName,\n            rolesConfigured: true\n        };\n        \n    } catch (error) {\n        console.error(\"\\n❌ Role setup failed:\");\n        console.error(\"Error:\", error.message);\n        \n        return {\n            success: false,\n            error: error.message,\n            contractAddress,\n            networkName\n        };\n    }\n}\n\n// Default role configuration for different environments\nconst defaultRoleConfigs = {\n    development: {\n        additionalRoles: [],\n        emergencyContacts: [],\n        operators: []\n    },\n    \n    testnet: {\n        additionalRoles: [\n            {\n                role: \"OPERATOR\",\n                addresses: [] // Add testnet operator addresses here\n            }\n        ],\n        emergencyContacts: [], // Add emergency contact addresses\n        operators: [] // Add operator addresses\n    },\n    \n    mainnet: {\n        additionalRoles: [\n            {\n                role: \"OPERATOR\",\n                addresses: [] // Add mainnet operator addresses here\n            }\n        ],\n        emergencyContacts: [], // Add emergency contact addresses\n        operators: [] // Add operator addresses\n    }\n};\n\n// Main execution\nasync function main() {\n    const args = process.argv.slice(2);\n    \n    if (args.length < 1) {\n        console.log(\"Usage: npx hardhat run scripts/setup-roles.js --network <network> <contract-address> [config-type]\");\n        console.log(\"Config types: development, testnet, mainnet\");\n        process.exit(1);\n    }\n    \n    const contractAddress = args[0];\n    const configType = args[1] || \"development\";\n    const networkName = process.env.HARDHAT_NETWORK || \"hardhat\";\n    \n    console.log(`Setting up roles for contract: ${contractAddress}`);\n    console.log(`Network: ${networkName}`);\n    console.log(`Config type: ${configType}`);\n    \n    const roleConfig = defaultRoleConfigs[configType];\n    if (!roleConfig) {\n        console.error(`Unknown config type: ${configType}`);\n        console.log(\"Available types:\", Object.keys(defaultRoleConfigs).join(\", \"));\n        process.exit(1);\n    }\n    \n    const result = await setupRoles(contractAddress, networkName, roleConfig);\n    \n    if (!result.success) {\n        process.exit(1);\n    }\n}\n\n// Export for use in other scripts\nmodule.exports = {\n    setupRoles,\n    defaultRoleConfigs\n};\n\n// Run if called directly\nif (require.main === module) {\n    main()\n        .then(() => process.exit(0))\n        .catch((error) => {\n            console.error(\"Role setup script failed:\", error);\n            process.exit(1);\n        });\n}"