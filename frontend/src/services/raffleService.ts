import { apiRequest } from './api';
import { 
  Raffle, 
  RaffleFilters, 
  RaffleListResponse, 
  BoxPurchaseRequest,
  CreateRaffleRequest,
  RaffleStatistics
} from '../types/raffle';

export const raffleService = {
  // Get all raffles with filters and pagination
  getRaffles: async (
    filters?: RaffleFilters,
    page: number = 1,
    limit: number = 12
  ): Promise<RaffleListResponse> => {
    const params = new URLSearchParams({
      page: page.toString(),
      limit: limit.toString(),
    });

    if (filters) {
      if (filters.status) {
        filters.status.forEach(status => params.append('status', status));
      }
      if (filters.category) {
        filters.category.forEach(category => params.append('category', category));
      }
      if (filters.priceRange) {
        params.append('minPrice', filters.priceRange.min.toString());
        params.append('maxPrice', filters.priceRange.max.toString());
      }
      if (filters.sortBy) {
        params.append('sortBy', filters.sortBy);
      }
      if (filters.search) {
        params.append('search', filters.search);
      }
    }

    return apiRequest.get<RaffleListResponse>(`/raffles?${params.toString()}`);
  },

  // Get a specific raffle by ID
  getRaffle: async (raffleId: string): Promise<Raffle> => {
    return apiRequest.get<Raffle>(`/raffles/${raffleId}`);
  },

  // Create a new raffle (sellers only)
  createRaffle: async (raffleData: CreateRaffleRequest): Promise<Raffle> => {
    return apiRequest.post<Raffle>('/raffles', raffleData);
  },

  // Update a raffle (sellers only)
  updateRaffle: async (raffleId: string, updates: Partial<Raffle>): Promise<Raffle> => {
    return apiRequest.patch<Raffle>(`/raffles/${raffleId}`, updates);
  },

  // Delete a raffle (sellers only)
  deleteRaffle: async (raffleId: string): Promise<void> => {
    return apiRequest.delete(`/raffles/${raffleId}`);
  },

  // Purchase boxes in a raffle
  purchaseBoxes: async (raffleId: string, purchaseData: BoxPurchaseRequest): Promise<{
    transactionId: string;
    boxNumbers: number[];
    totalCost: number;
    paymentMethod: string;
  }> => {
    return apiRequest.post(`/raffles/${raffleId}/buy-boxes`, purchaseData);
  },

  // Get user's participation in raffles
  getUserParticipations: async (userId?: string): Promise<{
    active: Raffle[];
    completed: Raffle[];
    won: Raffle[];
  }> => {
    const endpoint = userId ? `/users/${userId}/participations` : '/users/me/participations';
    return apiRequest.get(endpoint);
  },

  // Get raffle statistics
  getRaffleStatistics: async (raffleId?: string): Promise<RaffleStatistics> => {
    const endpoint = raffleId ? `/raffles/${raffleId}/statistics` : '/raffles/statistics';
    return apiRequest.get<RaffleStatistics>(endpoint);
  },

  // Get popular raffles
  getPopularRaffles: async (limit: number = 6): Promise<Raffle[]> => {
    return apiRequest.get<Raffle[]>(`/raffles/popular?limit=${limit}`);
  },

  // Get ending soon raffles
  getEndingSoonRaffles: async (limit: number = 6): Promise<Raffle[]> => {
    return apiRequest.get<Raffle[]>(`/raffles/ending-soon?limit=${limit}`);
  },

  // Get recently completed raffles
  getRecentlyCompletedRaffles: async (limit: number = 6): Promise<Raffle[]> => {
    return apiRequest.get<Raffle[]>(`/raffles/recently-completed?limit=${limit}`);
  },

  // Get raffles by category
  getRafflesByCategory: async (category: string, limit: number = 12): Promise<Raffle[]> => {
    return apiRequest.get<Raffle[]>(`/raffles/category/${category}?limit=${limit}`);
  },

  // Search raffles
  searchRaffles: async (query: string, limit: number = 12): Promise<Raffle[]> => {
    return apiRequest.get<Raffle[]>(`/raffles/search?q=${encodeURIComponent(query)}&limit=${limit}`);
  },

  // Get raffle participants
  getRaffleParticipants: async (raffleId: string): Promise<{
    participants: Array<{
      userId: string;
      username: string;
      avatar?: string;
      boxNumbers: number[];
      purchasedAt: string;
    }>;
    totalParticipants: number;
    totalBoxesSold: number;
  }> => {
    return apiRequest.get(`/raffles/${raffleId}/participants`);
  },

  // Get raffle winners (for completed raffles)
  getRaffleWinners: async (raffleId: string): Promise<{
    winners: Array<{
      userId: string;
      username: string;
      avatar?: string;
      winnerIndex: number;
      selectedAt: string;
    }>;
    totalWinners: number;
  }> => {
    return apiRequest.get(`/raffles/${raffleId}/winners`);
  },

  // Cancel raffle participation (if allowed)
  cancelParticipation: async (raffleId: string): Promise<void> => {
    return apiRequest.delete(`/raffles/${raffleId}/participation`);
  },

  // Report a raffle
  reportRaffle: async (raffleId: string, reason: string, description?: string): Promise<void> => {
    return apiRequest.post(`/raffles/${raffleId}/report`, {
      reason,
      description,
    });
  },

  // Get raffle categories
  getCategories: async (): Promise<Array<{
    name: string;
    count: number;
    description?: string;
  }>> => {
    return apiRequest.get('/raffles/categories');
  },

  // Get raffle activity feed
  getActivityFeed: async (raffleId: string, limit: number = 20): Promise<Array<{
    id: string;
    type: 'box_purchased' | 'raffle_full' | 'winner_selected' | 'raffle_completed';
    userId?: string;
    username?: string;
    data: any;
    timestamp: string;
  }>> => {
    return apiRequest.get(`/raffles/${raffleId}/activity?limit=${limit}`);
  },

  // Subscribe to raffle notifications
  subscribeToRaffle: async (raffleId: string): Promise<void> => {
    return apiRequest.post(`/raffles/${raffleId}/subscribe`);
  },

  // Unsubscribe from raffle notifications
  unsubscribeFromRaffle: async (raffleId: string): Promise<void> => {
    return apiRequest.delete(`/raffles/${raffleId}/subscribe`);
  },
};