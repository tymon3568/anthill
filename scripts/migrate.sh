#!/bin/bash
# Database Migration Helper Script
# Simplifies common sqlx-cli migration tasks

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Check if DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    echo -e "${RED}Error: DATABASE_URL not set${NC}"
    echo "Please create a .env file with DATABASE_URL or export it"
    exit 1
fi

# Check if sqlx-cli is installed
if ! command -v sqlx &> /dev/null; then
    echo -e "${YELLOW}sqlx-cli not found. Installing...${NC}"
    cargo install sqlx-cli --no-default-features --features postgres
fi

# Function to display usage
usage() {
    echo -e "${GREEN}Database Migration Helper${NC}"
    echo ""
    echo "Usage: $0 [command] [arguments]"
    echo ""
    echo "Commands:"
    echo "  create <name>    Create a new migration file"
    echo "  run              Run all pending migrations"
    echo "  revert           Revert the last migration"
    echo "  status           Show migration status"
    echo "  reset            Drop database and run all migrations"
    echo "  setup            Create database if it doesn't exist"
    echo "  drop             Drop the database"
    echo ""
    echo "Examples:"
    echo "  $0 create add_products_table"
    echo "  $0 run"
    echo "  $0 status"
    exit 1
}

# Parse command
case "$1" in
    create)
        if [ -z "$2" ]; then
            echo -e "${RED}Error: Migration name required${NC}"
            usage
        fi
        echo -e "${GREEN}Creating migration: $2${NC}"
        sqlx migrate add "$2"
        ;;

    run)
        echo -e "${GREEN}Running migrations...${NC}"
        sqlx migrate run
        echo -e "${GREEN}✓ Migrations complete${NC}"
        ;;

    revert)
        echo -e "${YELLOW}Reverting last migration...${NC}"
        sqlx migrate revert
        echo -e "${GREEN}✓ Migration reverted${NC}"
        ;;

    status)
        echo -e "${GREEN}Migration status:${NC}"
        sqlx migrate info
        ;;

    reset)
        echo -e "${YELLOW}Warning: This will drop the database and run all migrations!${NC}"
        read -p "Are you sure? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            echo -e "${YELLOW}Dropping database...${NC}"
            sqlx database drop -y
            echo -e "${GREEN}Creating database...${NC}"
            sqlx database create
            echo -e "${GREEN}Running migrations...${NC}"
            sqlx migrate run
            echo -e "${GREEN}✓ Database reset complete${NC}"
        else
            echo -e "${YELLOW}Cancelled${NC}"
        fi
        ;;

    setup)
        echo -e "${GREEN}Setting up database...${NC}"
        sqlx database create
        echo -e "${GREEN}✓ Database created${NC}"
        ;;

    drop)
        echo -e "${YELLOW}Warning: This will drop the database!${NC}"
        read -p "Are you sure? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            sqlx database drop -y
            echo -e "${GREEN}✓ Database dropped${NC}"
        else
            echo -e "${YELLOW}Cancelled${NC}"
        fi
        ;;

    *)
        usage
        ;;
esac
