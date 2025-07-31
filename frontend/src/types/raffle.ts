export interface Item {
  id: string;
  title: string;
  description: string;
  price: number;
  category: string;
  imageUrls: string[];
  sellerId: string;
  createdAt: string;
  updatedAt: string;
}

export interface Participant {
  id: string;
  userId: string;
  raffleId: string;
  boxNumber: number;
  purchasedAt: string;
  user?: {
    id: string;
    email: string;
    username?: string;
    avatar?: string;
  };
}

export interface Winner {
  id: string;
  userId: string;
  raffleId: string;
  winnerIndex: number;
  selectedAt: string;
  user?: {
    id: string;
    email: string;
    username?: string;
    avatar?: string;
  };
}

export interface Raffle {
  id: string;
  itemId: string;
  sellerId: string;
  totalBoxes: number;
  boxPrice: number;
  totalWinners: number;
  boxesSold: number;
  status: 'active' | 'full' | 'completed' | 'cancelled';
  createdAt: string;
  updatedAt: string;
  endTime?: string;
  item: Item;
  participants?: Participant[];
  winners?: Winner[];
}

export interface RaffleFilters {
  status?: ('active' | 'full' | 'completed')[];
  category?: string[];
  priceRange?: {
    min: number;
    max: number;
  };
  sortBy?: 'newest' | 'oldest' | 'price_low' | 'price_high' | 'ending_soon' | 'most_popular';
  search?: string;
}

export interface RaffleListResponse {
  raffles: Raffle[];
  total: number;
  page: number;
  limit: number;
  totalPages: number;
}

export interface BoxPurchaseRequest {
  boxNumbers: number[];
  paymentMethod: 'credits' | 'card';
  paymentIntentId?: string; // For card payments
}

export interface CreateRaffleRequest {
  itemId: string;
  totalBoxes: number;
  boxPrice: number;
  totalWinners: number;
  endTime?: string;
}

export interface RaffleStatistics {
  totalRaffles: number;
  activeRaffles: number;
  completedRaffles: number;
  totalParticipants: number;
  totalRevenue: number;
  averageBoxPrice: number;
  popularCategories: Array<{
    category: string;
    count: number;
  }>;
}

export interface RaffleActivity {
  id: string;
  type: 'box_purchased' | 'raffle_full' | 'winner_selected' | 'raffle_completed';
  raffleId: string;
  userId?: string;
  username?: string;
  data: any;
  timestamp: string;
}

// WebSocket message types
export interface RaffleWebSocketMessage {
  type: 'box_purchased' | 'raffle_full' | 'winner_selected' | 'raffle_completed' | 'participant_joined' | 'participant_left';
  raffleId: string;
  data: any;
}

export interface BoxPurchasedData {
  boxNumbers: number[];
  participant: Participant;
  updatedRaffle?: Raffle;
}

export interface WinnerSelectedData {
  winners: Winner[];
  updatedRaffle?: Raffle;
}

export interface RaffleCompletedData {
  winners: Winner[];
  updatedRaffle: Raffle;
}

export interface ParticipantJoinedData {
  participant: Participant;
  activeUsers: number;
}

export interface ParticipantLeftData {
  participantId: string;
  activeUsers: number;
}