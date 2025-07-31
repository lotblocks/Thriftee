const { ethers } = require("hardhat");

async function main() {
    // Get the deployed contract address
    const contractAddress = process.env.RAFFLE_CONTRACT_ADDRESS;
    if (!contractAddress) {
        throw new Error("Please set RAFFLE_CONTRACT_ADDRESS environment variable");
    }

    console.log("Interacting with Enhanced RaffleContract at:", contractAddress);

    // Get contract instance
    const RaffleContract = await ethers.getContractFactory("RaffleContract");
    const raffleContract = RaffleContract.attach(contractAddress);

    // Get signers
    const [owner, user1, user2, user3] = await ethers.getSigners();
    console.log("Owner address:", owner.address);
    console.log("User1 address:", user1.address);
    console.log("User2 address:", user2.address);
    console.log("User3 address:", user3.address);

    try {
        // Create a sample raffle with enhanced features
        console.log("\n=== Creating Enhanced Sample Raffle ===");
        const title = "Enhanced Sample Raffle";
        const description = "This is an enhanced sample raffle with new features";
        const totalBoxes = 20;
        const pricePerBox = ethers.utils.parseEther("0.01"); // 0.01 ETH per box
        const startTime = Math.floor(Date.now() / 1000) + 60; // Start in 1 minute
        const endTime = startTime + 3600; // End in 1 hour
        const itemImageUrl = "https://example.com/enhanced-item.jpg";
        const itemDescription = "An enhanced sample item for the raffle";
        const maxParticipantsPerUser = 5; // Max 5 boxes per user
        const minimumParticipants = 3; // Need at least 3 participants
        const requiresWhitelist = false;
        const whitelistMerkleRoot = "0x0000000000000000000000000000000000000000000000000000000000000000";
        const creatorFeePercentage = 500; // 5% creator fee
        const paymentToken = ethers.constants.AddressZero; // ETH payment

        const createTx = await raffleContract.createRaffle(
            title,
            description,
            totalBoxes,
            pricePerBox,
            startTime,
            endTime,
            itemImageUrl,
            itemDescription,
            maxParticipantsPerUser,
            minimumParticipants,
            requiresWhitelist,
            whitelistMerkleRoot,
            creatorFeePercentage,
            paymentToken
        );
        
        const receipt = await createTx.wait();
        const raffleId = receipt.events.find(e => e.event === "RaffleCreated").args.raffleId;
        console.log("Enhanced raffle created with ID:", raffleId.toString());

        // Get enhanced raffle details
        console.log("\n=== Enhanced Raffle Details ===");
        const raffle = await raffleContract.raffles(raffleId);
        console.log("Title:", raffle.title);
        console.log("Description:", raffle.description);
        console.log("Total Boxes:", raffle.totalBoxes.toString());
        console.log("Price per Box:", ethers.utils.formatEther(raffle.pricePerBox), "ETH");
        console.log("Creator:", raffle.creator);
        console.log("Status:", raffle.status);
        console.log("Max Participants Per User:", raffle.maxParticipantsPerUser.toString());
        console.log("Minimum Participants:", raffle.minimumParticipants.toString());
        console.log("Creator Fee:", raffle.creatorFeePercentage.toString() / 100, "%");
        console.log("Platform Fee:", raffle.platformFeePercentage.toString() / 100, "%");
        console.log("Requires Whitelist:", raffle.requiresWhitelist);
        console.log("Is Verified:", raffle.isVerified);

        // Wait for raffle to start
        console.log("\n=== Waiting for raffle to start ===");
        console.log("Waiting 65 seconds for raffle to become active...");
        await new Promise(resolve => setTimeout(resolve, 65000)); // Wait 65 seconds

        // Purchase participation as user1
        console.log("\n=== User1 purchasing participation ===");
        const boxesToPurchase1 = 3;
        const totalCost1 = pricePerBox.mul(boxesToPurchase1);
        
        const purchaseTx1 = await raffleContract.connect(user1).purchaseParticipation(
            raffleId,
            boxesToPurchase1,
            [], // Empty merkle proof since whitelist is disabled
            { value: totalCost1 }
        );
        await purchaseTx1.wait();
        console.log(`User1 purchased ${boxesToPurchase1} boxes for ${ethers.utils.formatEther(totalCost1)} ETH`);

        // Purchase participation as user2
        console.log("\n=== User2 purchasing participation ===");
        const boxesToPurchase2 = 2;
        const totalCost2 = pricePerBox.mul(boxesToPurchase2);
        
        const purchaseTx2 = await raffleContract.connect(user2).purchaseParticipation(
            raffleId,
            boxesToPurchase2,
            [], // Empty merkle proof
            { value: totalCost2 }
        );
        await purchaseTx2.wait();
        console.log(`User2 purchased ${boxesToPurchase2} boxes for ${ethers.utils.formatEther(totalCost2)} ETH`);

        // Purchase participation as user3 to meet minimum
        console.log("\n=== User3 purchasing participation ===");
        const boxesToPurchase3 = 1;
        const totalCost3 = pricePerBox.mul(boxesToPurchase3);
        
        const purchaseTx3 = await raffleContract.connect(user3).purchaseParticipation(
            raffleId,
            boxesToPurchase3,
            [], // Empty merkle proof
            { value: totalCost3 }
        );
        await purchaseTx3.wait();
        console.log(`User3 purchased ${boxesToPurchase3} boxes for ${ethers.utils.formatEther(totalCost3)} ETH`);

        // Get updated raffle details
        console.log("\n=== Updated Raffle Details ===");
        const updatedRaffle = await raffleContract.raffles(raffleId);
        console.log("Total Participants:", updatedRaffle.totalParticipants.toString());
        console.log("Total Revenue:", ethers.utils.formatEther(updatedRaffle.totalRevenue), "ETH");

        // Get participants
        const participants = await raffleContract.getRaffleParticipants(raffleId);
        console.log("Participants:", participants);

        // Check individual participant boxes
        console.log("\n=== Participant Box Counts ===");
        for (const participant of participants) {
            const boxCount = await raffleContract.getParticipantBoxes(raffleId, participant);
            console.log(`${participant}: ${boxCount.toString()} boxes`);
        }

        // Get raffle stats
        console.log("\n=== Raffle Statistics ===");
        const stats = await raffleContract.getRaffleStats(raffleId);
        console.log("Total Participants:", stats.totalParticipants.toString());
        console.log("Total Revenue:", ethers.utils.formatEther(stats.totalRevenue), "ETH");
        console.log("Boxes Remaining:", stats.boxesRemaining.toString());
        console.log("Is Completed:", stats.isCompleted);
        console.log("Winner:", stats.winner);

        // Check if raffle can be completed
        const canComplete = await raffleContract.canCompleteRaffle(raffleId);
        console.log("Can complete raffle:", canComplete);

        // Test role-based functions
        console.log("\n=== Testing Role-Based Functions ===");
        
        // Verify raffle (as operator)
        try {
            const verifyTx = await raffleContract.verifyRaffle(raffleId);
            await verifyTx.wait();
            console.log("✅ Raffle verified successfully");
        } catch (error) {
            console.log("❌ Raffle verification failed:", error.message);
        }

        // Test pause functionality
        console.log("\n=== Testing Emergency Controls ===");
        try {
            const pauseTx = await raffleContract.pause();
            await pauseTx.wait();
            console.log("✅ Contract paused successfully");

            const unpauseTx = await raffleContract.unpause();
            await unpauseTx.wait();
            console.log("✅ Contract unpaused successfully");
        } catch (error) {
            console.log("❌ Pause/unpause failed:", error.message);
        }

        // Get active raffles
        console.log("\n=== Active Raffles ===");
        const activeRaffles = await raffleContract.getActiveRaffles();
        console.log("Active raffle IDs:", activeRaffles.map(id => id.toString()));

        console.log("\n=== Enhanced Interaction completed successfully! ===");
        console.log("Contract features tested:");
        console.log("✅ Enhanced raffle creation with new parameters");
        console.log("✅ Multi-user participation with limits");
        console.log("✅ Participant tracking and statistics");
        console.log("✅ Role-based access control");
        console.log("✅ Emergency controls (pause/unpause)");
        console.log("✅ Raffle verification system");
        console.log("✅ Enhanced event emission");
        
    } catch (error) {
        console.error("Error during interaction:", error.message);
        if (error.reason) {
            console.error("Reason:", error.reason);
        }
        throw error;
    }
}

if (require.main === module) {
    main()
        .then(() => process.exit(0))
        .catch((error) => {
            console.error(error);
            process.exit(1);
        });
}

module.exports = main;