import { useEffect, useCallback, useState } from 'react';
import { toast } from 'react-toastify';

import { useWebSocket } from './useWebSocket';
import { Raffle, Participant } from '../types/raffle';
import { useAppDispatch } from '../store';
import { updateRaffle } from '../store/slices/raffleSlice';

interface RaffleRealTimeMessage {
  type: 'box_purchased' | 'raffle_full' | 'winner_selected' | 'raffle_completed' | 'participant_joined' | 'participant_left';
  raffleId: string;
  data: any;
}

interface UseRaffleRealTimeProps {
  raffleId: string;
  onBoxPurchased?: (data: { boxNumbers: number[]; participant: Participant }) => void;
  onRaffleFull?: () => void;
  onWinnerSelected?: (data: { winners: any[] }) => void;
  onRaffleCompleted?: () => void;
  onParticipantJoined?: (participant: Participant) => void;
  onParticipantLeft?: (participantId: string) => void;
}

interface UseRaffleRealTimeReturn {
  isConnected: boolean;
  isConnecting: boolean;
  activeUsers: number;
  joinRaffleRoom: () => void;
  leaveRaffleRoom: () => void;
}

export const useRaffleRealTime = ({
  raffleId,
  onBoxPurchased,
  onRaffleFull,
  onWinnerSelected,
  onRaffleCompleted,
  onParticipantJoined,
  onParticipantLeft,
}: UseRaffleRealTimeProps): UseRaffleRealTimeReturn => {
  const dispatch = useAppDispatch();
  const [activeUsers, setActiveUsers] = useState(0);
  const [hasJoinedRoom, setHasJoinedRoom] = useState(false);

  // Handle WebSocket messages
  const handleMessage = useCallback((message: any) => {
    const { type, raffleId: messageRaffleId, data } = message as RaffleRealTimeMessage;

    // Only process messages for our raffle
    if (messageRaffleId !== raffleId) return;

    switch (type) {
      case 'box_purchased':
        console.log('Box purchased:', data);
        onBoxPurchased?.(data);
        
        // Show toast notification
        toast.success(
          `${data.participant.user?.email || 'Someone'} purchased ${data.boxNumbers.length} box${data.boxNumbers.length > 1 ? 'es' : ''}!`,
          { autoClose: 3000 }
        );

        // Update Redux store if we have the full raffle data
        if (data.updatedRaffle) {
          dispatch(updateRaffle(data.updatedRaffle));
        }
        break;

      case 'raffle_full':
        console.log('Raffle is full:', data);
        onRaffleFull?.();
        
        toast.info('🎉 Raffle is now full! Winner selection will begin shortly.', {
          autoClose: 5000,
        });
        break;

      case 'winner_selected':
        console.log('Winner selected:', data);
        onWinnerSelected?.(data);
        
        const winnerNames = data.winners.map((w: any) => w.user?.email || 'Anonymous').join(', ');
        toast.success(`🏆 Winners selected: ${winnerNames}`, {
          autoClose: 10000,
        });
        break;

      case 'raffle_completed':
        console.log('Raffle completed:', data);
        onRaffleCompleted?.();
        
        toast.success('🎊 Raffle has been completed! Check your dashboard for results.', {
          autoClose: 10000,
        });

        // Update Redux store
        if (data.updatedRaffle) {
          dispatch(updateRaffle(data.updatedRaffle));
        }
        break;

      case 'participant_joined':
        console.log('Participant joined:', data);
        onParticipantJoined?.(data.participant);
        setActiveUsers(data.activeUsers || 0);
        break;

      case 'participant_left':
        console.log('Participant left:', data);
        onParticipantLeft?.(data.participantId);
        setActiveUsers(data.activeUsers || 0);
        break;

      default:
        console.log('Unknown message type:', type, data);
    }
  }, [raffleId, onBoxPurchased, onRaffleFull, onWinnerSelected, onRaffleCompleted, onParticipantJoined, onParticipantLeft, dispatch]);

  // Handle connection events
  const handleConnect = useCallback(() => {
    console.log('Connected to raffle WebSocket');
    // Auto-join the raffle room when connected
    if (!hasJoinedRoom) {
      joinRaffleRoom();
    }
  }, [hasJoinedRoom]);

  const handleDisconnect = useCallback(() => {
    console.log('Disconnected from raffle WebSocket');
    setHasJoinedRoom(false);
    setActiveUsers(0);
  }, []);

  const handleError = useCallback((error: Event) => {
    console.error('Raffle WebSocket error:', error);
    toast.error('Connection error. Real-time updates may not work properly.');
  }, []);

  // Initialize WebSocket connection
  const { isConnected, isConnecting, sendMessage } = useWebSocket('/ws/raffles', {
    onMessage: handleMessage,
    onConnect: handleConnect,
    onDisconnect: handleDisconnect,
    onError: handleError,
    reconnectAttempts: 10,
    reconnectInterval: 2000,
  });

  // Join raffle room
  const joinRaffleRoom = useCallback(() => {
    if (isConnected && !hasJoinedRoom) {
      sendMessage('join_raffle', { raffleId });
      setHasJoinedRoom(true);
      console.log('Joined raffle room:', raffleId);
    }
  }, [isConnected, hasJoinedRoom, sendMessage, raffleId]);

  // Leave raffle room
  const leaveRaffleRoom = useCallback(() => {
    if (isConnected && hasJoinedRoom) {
      sendMessage('leave_raffle', { raffleId });
      setHasJoinedRoom(false);
      setActiveUsers(0);
      console.log('Left raffle room:', raffleId);
    }
  }, [isConnected, hasJoinedRoom, sendMessage, raffleId]);

  // Auto-join room when connected
  useEffect(() => {
    if (isConnected && !hasJoinedRoom) {
      joinRaffleRoom();
    }
  }, [isConnected, hasJoinedRoom, joinRaffleRoom]);

  // Leave room on unmount
  useEffect(() => {
    return () => {
      if (hasJoinedRoom) {
        leaveRaffleRoom();
      }
    };
  }, [hasJoinedRoom, leaveRaffleRoom]);

  return {
    isConnected,
    isConnecting,
    activeUsers,
    joinRaffleRoom,
    leaveRaffleRoom,
  };
};