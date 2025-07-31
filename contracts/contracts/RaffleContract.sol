// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@chainlink/contracts/src/v0.8/vrf/VRFConsumerBaseV2.sol";
import "@chainlink/contracts/src/v0.8/interfaces/VRFCoordinatorV2Interface.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "@openzeppelin/contracts/utils/math/SafeMath.sol";

/**
 * @title RaffleContract
 * @dev Smart contract for managing provably fair raffles using Chainlink VRF
 * @author Unit Shopping Platform
 */
contract RaffleContract is VRFConsumerBaseV2, Ownable, ReentrancyGuard, Pausable {
    using SafeMath for uint256;

    // Chainlink VRF Configuration
    VRFCoordinatorV2Interface private immutable i_vrfCoordinator;
    bytes32 private immutable i_keyHash;
    uint64 private immutable i_subscriptionId;
    uint32 private constant CALLBACK_GAS_LIMIT = 100000;
    uint16 private constant REQUEST_CONFIRMATIONS = 3;

    // Raffle States
    enum RaffleStatus {
        OPEN,
        FULL,
        RANDOM_REQUESTED,
        COMPLETED,
        CANCELLED
    }

    // Raffle Structure
    struct Raffle {
        uint256 itemId;
        uint256 totalBoxes;
        uint256 boxPrice;
        uint256 boxesSold;
        uint256 totalWinners;
        address[] winnerAddresses;
        RaffleStatus status;
        uint256 requestId;
        uint256 randomWord;
        address creator;
        uint256 createdAt;
        uint256 completedAt;
    }

    // State Variables
    uint256 private s_raffleIdCounter;
    mapping(uint256 => Raffle) public s_raffles;
    mapping(uint256 => address[]) public s_boxOwners;
    mapping(uint256 => uint256) private s_requestIdToRaffleId;
    mapping(address => bool) public s_authorizedCallers;

    // Events
    event RaffleCreated(
        uint256 indexed raffleId,
        uint256 indexed itemId,
        uint256 totalBoxes,
        uint256 boxPrice,
        uint256 totalWinners,
        address indexed creator
    );

    event BoxPurchased(
        uint256 indexed raffleId,
        address indexed buyer,
        uint256 boxNumber,
        uint256 totalBoxesSold
    );

    event RaffleFull(uint256 indexed raffleId, uint256 totalBoxes);

    event RandomnessRequested(
        uint256 indexed raffleId,
        uint256 indexed requestId
    );

    event WinnerSelected(
        uint256 indexed raffleId,
        address[] winners,
        uint256 randomWord
    );

    event RaffleCancelled(uint256 indexed raffleId, string reason);

    event AuthorizedCallerAdded(address indexed caller);
    event AuthorizedCallerRemoved(address indexed caller);

    // Custom Errors
    error RaffleContract__InvalidRaffleId();
    error RaffleContract__RaffleNotOpen();
    error RaffleContract__RaffleFull();
    error RaffleContract__InvalidBoxPrice();
    error RaffleContract__InvalidWinnerCount();
    error RaffleContract__UnauthorizedCaller();
    error RaffleContract__RandomnessAlreadyRequested();
    error RaffleContract__InvalidRandomnessRequest();

    // Modifiers
    modifier onlyAuthorizedCaller() {
        if (!s_authorizedCallers[msg.sender] && msg.sender != owner()) {
            revert RaffleContract__UnauthorizedCaller();
        }
        _;
    }

    modifier validRaffleId(uint256 raffleId) {
        if (raffleId >= s_raffleIdCounter) {
            revert RaffleContract__InvalidRaffleId();
        }
        _;
    }

    modifier raffleInStatus(uint256 raffleId, RaffleStatus expectedStatus) {
        if (s_raffles[raffleId].status != expectedStatus) {
            revert RaffleContract__RaffleNotOpen();
        }
        _;
    }

    /**
     * @dev Constructor
     * @param vrfCoordinator Chainlink VRF Coordinator address
     * @param keyHash Chainlink VRF Key Hash
     * @param subscriptionId Chainlink VRF Subscription ID
     */
    constructor(
        address vrfCoordinator,
        bytes32 keyHash,
        uint64 subscriptionId
    ) VRFConsumerBaseV2(vrfCoordinator) {
        i_vrfCoordinator = VRFCoordinatorV2Interface(vrfCoordinator);
        i_keyHash = keyHash;
        i_subscriptionId = subscriptionId;
        s_raffleIdCounter = 0;
    }

    /**
     * @dev Create a new raffle
     * @param itemId Unique identifier for the item
     * @param totalBoxes Total number of boxes in the raffle
     * @param boxPrice Price per box (not used for validation, just stored)
     * @param totalWinners Number of winners to select
     */
    function createRaffle(
        uint256 itemId,
        uint256 totalBoxes,
        uint256 boxPrice,
        uint256 totalWinners
    ) external onlyAuthorizedCaller whenNotPaused returns (uint256) {
        if (totalBoxes == 0) revert RaffleContract__InvalidBoxPrice();
        if (totalWinners == 0 || totalWinners > totalBoxes) {
            revert RaffleContract__InvalidWinnerCount();
        }

        uint256 raffleId = s_raffleIdCounter;
        s_raffleIdCounter = s_raffleIdCounter.add(1);

        s_raffles[raffleId] = Raffle({
            itemId: itemId,
            totalBoxes: totalBoxes,
            boxPrice: boxPrice,
            boxesSold: 0,
            totalWinners: totalWinners,
            winnerAddresses: new address[](0),
            status: RaffleStatus.OPEN,
            requestId: 0,
            randomWord: 0,
            creator: msg.sender,
            createdAt: block.timestamp,
            completedAt: 0
        });

        emit RaffleCreated(
            raffleId,
            itemId,
            totalBoxes,
            boxPrice,
            totalWinners,
            msg.sender
        );

        return raffleId;
    }

    /**
     * @dev Purchase a box in the raffle
     * @param raffleId ID of the raffle
     */
    function buyBox(uint256 raffleId)
        external
        onlyAuthorizedCaller
        whenNotPaused
        validRaffleId(raffleId)
        raffleInStatus(raffleId, RaffleStatus.OPEN)
        nonReentrant
    {
        Raffle storage raffle = s_raffles[raffleId];

        if (raffle.boxesSold >= raffle.totalBoxes) {
            revert RaffleContract__RaffleFull();
        }

        // Record the box purchase
        s_boxOwners[raffleId].push(msg.sender);
        raffle.boxesSold = raffle.boxesSold.add(1);

        emit BoxPurchased(
            raffleId,
            msg.sender,
            raffle.boxesSold,
            raffle.boxesSold
        );

        // Check if raffle is now full
        if (raffle.boxesSold == raffle.totalBoxes) {
            raffle.status = RaffleStatus.FULL;
            emit RaffleFull(raffleId, raffle.totalBoxes);

            // Automatically request randomness
            _requestRandomness(raffleId);
        }
    }

    /**
     * @dev Request randomness from Chainlink VRF (internal function)
     * @param raffleId ID of the raffle
     */
    function _requestRandomness(uint256 raffleId) internal {
        Raffle storage raffle = s_raffles[raffleId];

        if (raffle.status != RaffleStatus.FULL) {
            revert RaffleContract__RandomnessAlreadyRequested();
        }

        uint256 requestId = i_vrfCoordinator.requestRandomWords(
            i_keyHash,
            i_subscriptionId,
            REQUEST_CONFIRMATIONS,
            CALLBACK_GAS_LIMIT,
            1 // numWords
        );

        raffle.requestId = requestId;
        raffle.status = RaffleStatus.RANDOM_REQUESTED;
        s_requestIdToRaffleId[requestId] = raffleId;

        emit RandomnessRequested(raffleId, requestId);
    }

    /**
     * @dev Callback function used by VRF Coordinator
     * @param requestId ID of the VRF request
     * @param randomWords Array of random words from Chainlink VRF
     */
    function fulfillRandomWords(uint256 requestId, uint256[] memory randomWords)
        internal
        override
    {
        uint256 raffleId = s_requestIdToRaffleId[requestId];
        Raffle storage raffle = s_raffles[raffleId];

        if (raffle.status != RaffleStatus.RANDOM_REQUESTED) {
            revert RaffleContract__InvalidRandomnessRequest();
        }

        uint256 randomWord = randomWords[0];
        raffle.randomWord = randomWord;

        // Select winners using the random word
        address[] memory winners = _selectWinners(raffleId, randomWord);
        raffle.winnerAddresses = winners;
        raffle.status = RaffleStatus.COMPLETED;
        raffle.completedAt = block.timestamp;

        emit WinnerSelected(raffleId, winners, randomWord);
    }

    /**
     * @dev Select winners using the random word
     * @param raffleId ID of the raffle
     * @param randomWord Random word from Chainlink VRF
     */
    function _selectWinners(uint256 raffleId, uint256 randomWord)
        internal
        view
        returns (address[] memory)
    {
        Raffle storage raffle = s_raffles[raffleId];
        address[] memory boxOwners = s_boxOwners[raffleId];
        address[] memory winners = new address[](raffle.totalWinners);

        // Use a simple but fair selection algorithm
        // For multiple winners, we derive additional random numbers from the original
        for (uint256 i = 0; i < raffle.totalWinners; i++) {
            uint256 derivedRandom = uint256(
                keccak256(abi.encode(randomWord, i, block.timestamp))
            );
            uint256 winnerIndex = derivedRandom % boxOwners.length;

            // Ensure we don't select the same winner twice
            address selectedWinner = boxOwners[winnerIndex];
            bool alreadySelected = false;

            for (uint256 j = 0; j < i; j++) {
                if (winners[j] == selectedWinner) {
                    alreadySelected = true;
                    break;
                }
            }

            if (!alreadySelected) {
                winners[i] = selectedWinner;
            } else {
                // Find next available winner
                for (uint256 k = 0; k < boxOwners.length; k++) {
                    uint256 nextIndex = (winnerIndex + k) % boxOwners.length;
                    address nextWinner = boxOwners[nextIndex];
                    bool nextAlreadySelected = false;

                    for (uint256 j = 0; j < i; j++) {
                        if (winners[j] == nextWinner) {
                            nextAlreadySelected = true;
                            break;
                        }
                    }

                    if (!nextAlreadySelected) {
                        winners[i] = nextWinner;
                        break;
                    }
                }
            }
        }

        return winners;
    }

    /**
     * @dev Cancel a raffle (emergency function)
     * @param raffleId ID of the raffle to cancel
     * @param reason Reason for cancellation
     */
    function cancelRaffle(uint256 raffleId, string calldata reason)
        external
        onlyOwner
        validRaffleId(raffleId)
    {
        Raffle storage raffle = s_raffles[raffleId];
        
        if (raffle.status == RaffleStatus.COMPLETED) {
            revert RaffleContract__InvalidRaffleId();
        }

        raffle.status = RaffleStatus.CANCELLED;
        emit RaffleCancelled(raffleId, reason);
    }

    /**
     * @dev Add authorized caller
     * @param caller Address to authorize
     */
    function addAuthorizedCaller(address caller) external onlyOwner {
        s_authorizedCallers[caller] = true;
        emit AuthorizedCallerAdded(caller);
    }

    /**
     * @dev Remove authorized caller
     * @param caller Address to remove authorization
     */
    function removeAuthorizedCaller(address caller) external onlyOwner {
        s_authorizedCallers[caller] = false;
        emit AuthorizedCallerRemoved(caller);
    }

    /**
     * @dev Pause the contract
     */
    function pause() external onlyOwner {
        _pause();
    }

    /**
     * @dev Unpause the contract
     */
    function unpause() external onlyOwner {
        _unpause();
    }

    // View Functions

    /**
     * @dev Get raffle details
     * @param raffleId ID of the raffle
     */
    function getRaffle(uint256 raffleId)
        external
        view
        validRaffleId(raffleId)
        returns (Raffle memory)
    {
        return s_raffles[raffleId];
    }

    /**
     * @dev Get winners of a raffle
     * @param raffleId ID of the raffle
     */
    function getWinners(uint256 raffleId)
        external
        view
        validRaffleId(raffleId)
        returns (address[] memory)
    {
        return s_raffles[raffleId].winnerAddresses;
    }

    /**
     * @dev Get box owners of a raffle
     * @param raffleId ID of the raffle
     */
    function getBoxOwners(uint256 raffleId)
        external
        view
        validRaffleId(raffleId)
        returns (address[] memory)
    {
        return s_boxOwners[raffleId];
    }

    /**
     * @dev Get current raffle counter
     */
    function getRaffleCounter() external view returns (uint256) {
        return s_raffleIdCounter;
    }

    /**
     * @dev Check if address is authorized caller
     * @param caller Address to check
     */
    function isAuthorizedCaller(address caller) external view returns (bool) {
        return s_authorizedCallers[caller] || caller == owner();
    }

    /**
     * @dev Get VRF configuration
     */
    function getVRFConfig()
        external
        view
        returns (
            address vrfCoordinator,
            bytes32 keyHash,
            uint64 subscriptionId
        )
    {
        return (address(i_vrfCoordinator), i_keyHash, i_subscriptionId);
    }
}