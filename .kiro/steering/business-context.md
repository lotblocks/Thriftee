# Business Context and Domain Knowledge

## Platform Overview

The Unit Shopping Platform is a revolutionary e-commerce platform that combines traditional online shopping with gamification through a raffle-based system. The platform creates an engaging, transparent, and fair shopping experience where users purchase "boxes" or "units" of items using credits, with winners selected through verifiable blockchain-based randomness.

## Core Business Model

### Revenue Streams
1. **Seller Subscription Fees**: Monthly recurring revenue from sellers based on subscription tiers
2. **Listing Fees**: Fees charged when sellers list items (varies by subscription tier)
3. **Transaction Fees**: Percentage-based fees on successful sales (varies by subscription tier)
4. **Credit Purchase Margins**: Small margin on credit purchases through payment processing

### Value Propositions

#### For Buyers
- **No Financial Loss Guarantee**: Credits ensure users never lose money
- **Engaging Experience**: Interactive visual grid creates excitement and community
- **Transparent Process**: Blockchain-based winner selection ensures fairness
- **Flexible Redemption**: Multiple options for using credits and recovering value

#### For Sellers
- **Guaranteed Sales**: Items sell when all boxes are purchased
- **Reduced Risk**: Platform handles payment processing and fraud prevention
- **Marketing Benefits**: Raffle format creates buzz and engagement
- **Flexible Pricing**: Can set profitable box prices based on item value

#### For Platform
- **Predictable Revenue**: Subscription and fee-based model
- **High Engagement**: Gamification increases user retention
- **Scalable Model**: Technology stack supports growth
- **Trust and Transparency**: Blockchain integration builds user confidence

## Key Business Rules

### Credit System Rules
- Credits are non-transferable between users
- Credits have expiration dates to encourage platform engagement
- Expiring credits can be redeemed for free items to ensure no loss
- Item-specific credits can only be used for the original item or similar items
- General credits can be used platform-wide

### Raffle Mechanics
- All boxes must be sold before winner selection begins
- Winner selection uses Chainlink VRF for verifiable randomness
- Multiple winners can be configured per raffle
- Non-winners receive credits equal to their box purchase value
- Winners receive the physical item and shipping

### Seller Requirements
- Sellers must maintain active subscriptions to list items
- Items must meet quality and description standards
- Sellers are responsible for shipping won items
- Platform takes fees only on successful sales
- Sellers can track performance through comprehensive dashboards

## Financial Model

### Profitability Formula
For each raffle to be profitable:
```
Total Revenue (N_boxes × P_box) ≥ COGS + Operating Costs + Target Profit
```

Where:
- N_boxes: Total number of boxes/units
- P_box: Price per box
- COGS: Cost of goods sold (item cost + shipping)
- Operating Costs: Platform costs (VRF fees, processing, etc.)
- Target Profit: Desired profit margin

### Credit Liability Management
- Credits represent future discounts, not immediate cash outflows
- Track credit redemption rates to understand true cost
- Offer free item redemption for expiring credits
- Maintain healthy margins on credit-redeemable items

## User Personas

### Primary Buyer Persona: "The Thrill Seeker"
- Age: 25-40
- Enjoys gamification and excitement in shopping
- Values transparency and fairness
- Comfortable with technology and mobile apps
- Seeks unique and interesting products

### Secondary Buyer Persona: "The Value Hunter"
- Age: 30-50
- Attracted to potential savings and deals
- Risk-averse but interested in no-loss guarantee
- Prefers detailed information and transparency
- Values customer service and support

### Seller Persona: "The Entrepreneur"
- Small to medium business owner
- Seeks new sales channels and marketing opportunities
- Values predictable costs and transparent fee structures
- Needs tools for inventory and business management
- Wants detailed analytics and reporting

## Competitive Landscape

### Direct Competitors
- Traditional raffle/lottery platforms (but without e-commerce focus)
- Gamified shopping apps (but without blockchain transparency)
- Auction sites (but with different mechanics)

### Competitive Advantages
- **Blockchain Transparency**: Verifiable fairness through smart contracts
- **No-Loss Guarantee**: Unique credit system eliminates financial risk
- **Interactive Experience**: Visual grid system creates engagement
- **Multi-Stakeholder Platform**: Serves buyers, sellers, and operators

### Market Differentiation
- First platform to combine raffle mechanics with guaranteed value recovery
- Blockchain-based transparency in winner selection
- Interactive visual experience during raffle participation
- Comprehensive business tools for sellers

## Regulatory Considerations

### Gambling Regulations
- Platform avoids gambling classification through no-loss guarantee
- Credits ensure users can always recover equivalent value
- Transparent odds and processes reduce regulatory risk
- Legal review required in each operating jurisdiction

### Financial Regulations
- PCI DSS compliance for payment processing
- AML/KYC requirements for high-value transactions
- Consumer protection laws compliance
- Data privacy regulations (GDPR, CCPA)

### E-commerce Regulations
- Consumer rights and return policies
- Product liability and safety standards
- Advertising and marketing compliance
- Cross-border trade regulations

## Success Metrics

### User Engagement
- Daily/Monthly Active Users (DAU/MAU)
- Average session duration
- Raffle participation rate
- Credit redemption rate
- User retention rate

### Business Performance
- Monthly Recurring Revenue (MRR) from subscriptions
- Average Revenue Per User (ARPU)
- Seller acquisition and retention rates
- Platform take rate (fees as % of GMV)
- Credit liability ratio

### Platform Health
- Raffle completion rate (% of raffles that fill all boxes)
- Average time to fill raffles
- User satisfaction scores
- Seller satisfaction scores
- Platform uptime and performance metrics

## Growth Strategy

### Phase 1: MVP Launch
- Focus on core raffle functionality
- Limited seller onboarding
- Basic credit system implementation
- Essential user features

### Phase 2: Scale and Optimize
- Advanced seller tools and analytics
- Enhanced user experience features
- Mobile app development
- International expansion preparation

### Phase 3: Platform Expansion
- Additional game modes and raffle types
- Social features and community building
- Advanced analytics and AI recommendations
- Partnership integrations

## Risk Management

### Technical Risks
- Smart contract vulnerabilities
- Blockchain network congestion
- Payment processing failures
- System scalability challenges

### Business Risks
- Regulatory changes affecting operations
- Competition from established players
- User adoption challenges
- Seller acquisition difficulties

### Mitigation Strategies
- Comprehensive security audits and testing
- Multiple blockchain network support
- Diversified payment processing options
- Strong legal and compliance framework
- Continuous user feedback and iteration