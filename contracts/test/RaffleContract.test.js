const { expect } = require("chai");
const { ethers } = require("hardhat");
const { loadFixture } = require("@nomicfoundation/hardhat-network-helpers");

describe("RaffleContract", function () {
  // Fixture to deploy the contract
  async function deployRaffleContractFixture() {
    const [owner, operator, user1, user2, user3, user4] = await ethers.getSigners();

    // Mock VRF Coordinator for testing
    const MockVRFCoordinator = await ethers.getContractFactory("MockVRFCoordinator");
    const mockVRFCoordinator = await MockVRFCoordinator.deploy();

    // Deploy RaffleContract
    const RaffleContract = await ethers.getContractFactory("RaffleContract");
    const subscriptionId = 1;
    const keyHash = "0x4b09e658ed251bcafeebbc69400383d49f344ace09b9576fe248bb02c003fe9f";
    
    const raffleContract = await RaffleContract.deploy(
      await mockVRFCoordinator.getAddress(),
      subscriptionId,
      keyHash
    );

    // Grant operator role
    const OPERATOR_ROLE = await raffleContract.OPERATOR_ROLE();
    await raffleContract.grantRole(OPERATOR_ROLE, operator.address);

    // Add operator as authorized caller
    await raffleContract.connect(operator).addAuthorizedCaller(operator.address);

    return {
      raffleContract,
      mockVRFCoordinator,
      owner,
      operator,
      user1,
      user2,
      user3,
      user4,
      subscriptionId,
      keyHash,
    };
  }

  describe("Deployment", function () {
    it("Should deploy with correct initial values", async function () {
      const { raffleContract, owner, subscriptionId, keyHash } = await loadFixture(
        deployRaffleContractFixture
      );

      expect(await raffleContract.getTotalRaffles()).to.equal(0);
      expect(await raffleContract.hasRole(await raffleContract.DEFAULT_ADMIN_ROLE(), owner.address)).to.be.true;
      expect(await raffleContract.hasRole(await raffleContract.OPERATOR_ROLE(), owner.address)).to.be.true;
    });
  });

  describe("Raffle Creation", function () {
    it("Should create a raffle with valid parameters", async function () {
      const { raffleContract, operator } = await loadFixture(deployRaffleContractFixture);

      const itemId = 1;
      const totalBoxes = 100;
      const boxPrice = ethers.parseEther("0.01");
      const totalWinners = 1;

      await expect(
        raffleContract.connect(operator).createRaffle(itemId, totalBoxes, boxPrice, totalWinners)
      )
        .to.emit(raffleContract, "RaffleCreated")
        .withArgs(0, itemId, totalBoxes, boxPrice, totalWinners, operator.address);

      const raffle = await raffleContract.getRaffle(0);
      expect(raffle.itemId).to.equal(itemId);
      expect(raffle.totalBoxes).to.equal(totalBoxes);
      expect(raffle.boxPrice).to.equal(boxPrice);
      expect(raffle.totalWinners).to.equal(totalWinners);
      expect(raffle.boxesSold).to.equal(0);
      expect(raffle.status).to.equal(0); // OPEN
    });

    it("Should reject raffle creation with invalid parameters", async function () {
      const { raffleContract, operator } = await loadFixture(deployRaffleContractFixture);

      // Zero total boxes
      await expect(
        raffleContract.connect(operator).createRaffle(1, 0, ethers.parseEther("0.01"), 1)
      ).to.be.revertedWithCustomError(raffleContract, "InvalidParameters");

      // Zero box price
      await expect(
        raffleContract.connect(operator).createRaffle(1, 100, 0, 1)
      ).to.be.revertedWithCustomError(raffleContract, "InvalidParameters");

      // Zero winners
      await expect(
        raffleContract.connect(operator).createRaffle(1, 100, ethers.parseEther("0.01"), 0)
      ).to.be.revertedWithCustomError(raffleContract, "InvalidParameters");

      // More winners than boxes
      await expect(
        raffleContract.connect(operator).createRaffle(1, 10, ethers.parseEther("0.01"), 20)
      ).to.be.revertedWithCustomError(raffleContract, "InvalidParameters");
    });

    it("Should reject raffle creation from unauthorized caller", async function () {
      const { raffleContract, user1 } = await loadFixture(deployRaffleContractFixture);

      await expect(
        raffleContract.connect(user1).createRaffle(1, 100, ethers.parseEther("0.01"), 1)
      ).to.be.revertedWithCustomError(raffleContract, "UnauthorizedCaller");
    });
  });

  describe("Box Purchasing", function () {
    it("Should allow users to buy boxes", async function () {
      const { raffleContract, operator, user1 } = await loadFixture(deployRaffleContractFixture);

      // Create a raffle
      const boxPrice = ethers.parseEther("0.01");
      await raffleContract.connect(operator).createRaffle(1, 100, boxPrice, 1);

      // Buy a box
      await expect(
        raffleContract.connect(user1).buyBox(0, { value: boxPrice })
      )
        .to.emit(raffleContract, "BoxPurchased")
        .withArgs(0, user1.address, 1, 1);

      const raffle = await raffleContract.getRaffle(0);
      expect(raffle.boxesSold).to.equal(1);

      const boxOwners = await raffleContract.getBoxOwners(0);
      expect(boxOwners[0]).to.equal(user1.address);
    });

    it("Should reject box purchase with incorrect payment", async function () {
      const { raffleContract, operator, user1 } = await loadFixture(deployRaffleContractFixture);

      const boxPrice = ethers.parseEther("0.01");
      await raffleContract.connect(operator).createRaffle(1, 100, boxPrice, 1);

      // Try to buy with wrong amount
      await expect(
        raffleContract.connect(user1).buyBox(0, { value: ethers.parseEther("0.005") })
      ).to.be.revertedWithCustomError(raffleContract, "InsufficientPayment");
    });

    it("Should reject box purchase for non-existent raffle", async function () {
      const { raffleContract, user1 } = await loadFixture(deployRaffleContractFixture);

      await expect(
        raffleContract.connect(user1).buyBox(999, { value: ethers.parseEther("0.01") })
      ).to.be.revertedWithCustomError(raffleContract, "RaffleNotFound");
    });

    it("Should handle raffle completion when all boxes are sold", async function () {
      const { raffleContract, mockVRFCoordinator, operator, user1, user2 } = await loadFixture(
        deployRaffleContractFixture
      );

      const boxPrice = ethers.parseEther("0.01");
      await raffleContract.connect(operator).createRaffle(1, 2, boxPrice, 1);

      // Buy first box
      await raffleContract.connect(user1).buyBox(0, { value: boxPrice });

      // Buy second box - should trigger raffle completion
      await expect(
        raffleContract.connect(user2).buyBox(0, { value: boxPrice })
      )
        .to.emit(raffleContract, "BoxPurchased")
        .and.to.emit(raffleContract, "RaffleFull")
        .and.to.emit(raffleContract, "RandomnessRequested");

      const raffle = await raffleContract.getRaffle(0);
      expect(raffle.status).to.equal(2); // RANDOM_REQUESTED
    });
  });

  describe("Winner Selection", function () {
    it("Should select winners correctly", async function () {
      const { raffleContract, mockVRFCoordinator, operator, user1, user2 } = await loadFixture(
        deployRaffleContractFixture
      );

      const boxPrice = ethers.parseEther("0.01");
      await raffleContract.connect(operator).createRaffle(1, 2, boxPrice, 1);

      // Fill the raffle
      await raffleContract.connect(user1).buyBox(0, { value: boxPrice });
      await raffleContract.connect(user2).buyBox(0, { value: boxPrice });

      // Get the request ID and fulfill randomness
      const raffle = await raffleContract.getRaffle(0);
      const requestId = raffle.requestId;

      // Simulate VRF response
      const randomWord = 12345;
      await expect(
        mockVRFCoordinator.fulfillRandomWords(requestId, [randomWord])
      )
        .to.emit(raffleContract, "WinnerSelected");

      const updatedRaffle = await raffleContract.getRaffle(0);
      expect(updatedRaffle.status).to.equal(3); // COMPLETED
      expect(updatedRaffle.randomWord).to.equal(randomWord);

      const winners = await raffleContract.getWinners(0);
      expect(winners.length).to.equal(1);
      expect([user1.address, user2.address]).to.include(winners[0]);
    });
  });

  describe("Access Control", function () {
    it("Should allow admin to add/remove authorized callers", async function () {
      const { raffleContract, operator, user1 } = await loadFixture(deployRaffleContractFixture);

      // Add authorized caller
      await expect(
        raffleContract.connect(operator).addAuthorizedCaller(user1.address)
      )
        .to.emit(raffleContract, "AuthorizedCallerAdded")
        .withArgs(user1.address);

      expect(await raffleContract.isAuthorizedCaller(user1.address)).to.be.true;

      // Remove authorized caller
      await expect(
        raffleContract.connect(operator).removeAuthorizedCaller(user1.address)
      )
        .to.emit(raffleContract, "AuthorizedCallerRemoved")
        .withArgs(user1.address);

      expect(await raffleContract.isAuthorizedCaller(user1.address)).to.be.false;
    });

    it("Should allow operator to cancel raffles", async function () {
      const { raffleContract, operator } = await loadFixture(deployRaffleContractFixture);

      await raffleContract.connect(operator).createRaffle(1, 100, ethers.parseEther("0.01"), 1);

      await expect(
        raffleContract.connect(operator).cancelRaffle(0, "Test cancellation")
      )
        .to.emit(raffleContract, "RaffleCancelled")
        .withArgs(0, "Test cancellation");

      const raffle = await raffleContract.getRaffle(0);
      expect(raffle.status).to.equal(4); // CANCELLED
    });

    it("Should allow pauser to pause/unpause contract", async function () {
      const { raffleContract, owner } = await loadFixture(deployRaffleContractFixture);

      await raffleContract.connect(owner).pause();
      expect(await raffleContract.paused()).to.be.true;

      await raffleContract.connect(owner).unpause();
      expect(await raffleContract.paused()).to.be.false;
    });
  });

  describe("View Functions", function () {
    it("Should return correct raffle information", async function () {
      const { raffleContract, operator } = await loadFixture(deployRaffleContractFixture);

      const itemId = 1;
      const totalBoxes = 100;
      const boxPrice = ethers.parseEther("0.01");
      const totalWinners = 1;

      await raffleContract.connect(operator).createRaffle(itemId, totalBoxes, boxPrice, totalWinners);

      const raffle = await raffleContract.getRaffle(0);
      expect(raffle.itemId).to.equal(itemId);
      expect(raffle.totalBoxes).to.equal(totalBoxes);
      expect(raffle.boxPrice).to.equal(boxPrice);
      expect(raffle.totalWinners).to.equal(totalWinners);

      expect(await raffleContract.getTotalRaffles()).to.equal(1);
      expect(await raffleContract.getRaffleStatus(0)).to.equal(0); // OPEN
    });
  });

  describe("Emergency Functions", function () {
    it("Should allow admin to emergency withdraw", async function () {
      const { raffleContract, owner, operator, user1 } = await loadFixture(deployRaffleContractFixture);

      // Create raffle and buy box to add funds to contract
      const boxPrice = ethers.parseEther("0.01");
      await raffleContract.connect(operator).createRaffle(1, 100, boxPrice, 1);
      await raffleContract.connect(user1).buyBox(0, { value: boxPrice });

      const initialBalance = await ethers.provider.getBalance(owner.address);
      const contractBalance = await ethers.provider.getBalance(await raffleContract.getAddress());

      await raffleContract.connect(owner).emergencyWithdraw();

      const finalBalance = await ethers.provider.getBalance(owner.address);
      expect(finalBalance).to.be.gt(initialBalance);
    });
  });
});