// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@chainlink/contracts/src/v0.8/interfaces/VRFCoordinatorV2Interface.sol";

/**
 * @title MockVRFCoordinator
 * @dev Mock VRF Coordinator for testing purposes
 */
contract MockVRFCoordinator is VRFCoordinatorV2Interface {
    uint256 private requestIdCounter = 1;
    mapping(uint256 => address) private requestIdToConsumer;

    event RandomWordsRequested(
        bytes32 indexed keyHash,
        uint256 requestId,
        uint256 preSeed,
        uint64 indexed subId,
        uint16 minimumRequestConfirmations,
        uint32 callbackGasLimit,
        uint32 numWords,
        address indexed sender
    );

    function requestRandomWords(
        bytes32 keyHash,
        uint64 subId,
        uint16 minimumRequestConfirmations,
        uint32 callbackGasLimit,
        uint32 numWords
    ) external override returns (uint256 requestId) {
        requestId = requestIdCounter++;
        requestIdToConsumer[requestId] = msg.sender;

        emit RandomWordsRequested(
            keyHash,
            requestId,
            0, // preSeed
            subId,
            minimumRequestConfirmations,
            callbackGasLimit,
            numWords,
            msg.sender
        );

        return requestId;
    }

    function fulfillRandomWords(uint256 requestId, uint256[] memory randomWords) external {
        address consumer = requestIdToConsumer[requestId];
        require(consumer != address(0), "Request not found");

        // Call the consumer's fulfillRandomWords function
        (bool success, ) = consumer.call(
            abi.encodeWithSignature("rawFulfillRandomWords(uint256,uint256[])", requestId, randomWords)
        );
        require(success, "Callback failed");
    }

    // Required interface functions (not used in testing)
    function createSubscription() external pure override returns (uint64 subId) {
        return 1;
    }

    function requestSubscriptionOwnerTransfer(uint64, address) external pure override {}

    function acceptSubscriptionOwnerTransfer(uint64) external pure override {}

    function addConsumer(uint64, address) external pure override {}

    function removeConsumer(uint64, address) external pure override {}

    function cancelSubscription(uint64, address) external pure override {}

    function pendingRequestExists(uint64) external pure override returns (bool) {
        return false;
    }

    function getSubscription(uint64)
        external
        pure
        override
        returns (
            uint96 balance,
            uint64 reqCount,
            address owner,
            address[] memory consumers
        )
    {
        return (0, 0, address(0), new address[](0));
    }
}