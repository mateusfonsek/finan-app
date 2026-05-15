import IndexRedirect from "./IndexRedirect.svelte";
import Onboarding from "./Onboarding.svelte";
import Dashboard from "./Dashboard.svelte";
import Transactions from "./Transactions.svelte";
import Import from "./Import.svelte";
import Categories from "./Categories.svelte";
import Rules from "./Rules.svelte";
import Calendar from "./Calendar.svelte";
import Settings from "./Settings.svelte";

export const routes = {
  "/": IndexRedirect,
  "/onboarding": Onboarding,
  "/dashboard": Dashboard,
  "/transactions": Transactions,
  "/import": Import,
  "/categories": Categories,
  "/rules": Rules,
  "/calendar": Calendar,
  "/settings": Settings,
};
