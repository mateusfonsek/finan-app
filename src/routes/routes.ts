import Onboarding from "./Onboarding.svelte";
import Dashboard from "./Dashboard.svelte";
import Transactions from "./Transactions.svelte";
import Import from "./Import.svelte";
import Categories from "./Categories.svelte";
import Rules from "./Rules.svelte";
import Settings from "./Settings.svelte";

export const routes = {
  "/": Dashboard,
  "/onboarding": Onboarding,
  "/dashboard": Dashboard,
  "/transactions": Transactions,
  "/import": Import,
  "/categories": Categories,
  "/rules": Rules,
  "/settings": Settings,
};
