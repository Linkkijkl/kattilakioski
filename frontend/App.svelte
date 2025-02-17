<script lang="ts">
  import IconButton, { Icon } from "@smui/icon-button";
  import TopAppBar, {
    Row,
    Section,
    Title as TopAppBarTitle,
  } from "@smui/top-app-bar";
  import {
    mdiMenu,
    mdiSearchWeb,
    mdiBasket,
    mdiCash,
    mdiAccountCircle,
    mdiCrown,
  } from "@mdi/js";
  import Drawer, {
    AppContent,
    Content,
    Header,
    Title as DrawerTitle,
    Subtitle,
  } from "@smui/drawer";
  import List, { Item, Text, Graphic, Separator, Subheader } from "@smui/list";
  import ItemListing from "./lib/ItemListing.svelte";
  import Login from "./lib/User.svelte";
  import Sell from "./lib/Sell.svelte";
  import { updateAPI, userInfo } from "./api.svelte";
  import { onMount } from "svelte";
  import Admin from "./lib/Admin.svelte";

  let drawerOpen = $state(false);
  let view = $state("buy");
  const setView = (val: string) => (view = val);

  onMount(updateAPI);
</script>

<main>
  <Drawer variant="modal" bind:open={drawerOpen}>
    <Header>
      <DrawerTitle>Kattilakioski</DrawerTitle>
      <Subtitle></Subtitle>
    </Header>
    <Content>
      <List>
        <Item
          href="javascript:void(0)"
          onclick={() => setView("buy")}
          activated={view === "buy"}
        >
          <IconButton aria-label="Buy">
            <Icon tag="svg" viewBox="0 0 24 24">
              <path fill="currentColor" d={mdiBasket} />
            </Icon>
          </IconButton>
          <Text>Buy</Text>
        </Item>
        <Item
          href="javascript:void(0)"
          onclick={() => setView("sell")}
          activated={view === "sell"}
        >
          <IconButton aria-label="Sell">
            <Icon tag="svg" viewBox="0 0 24 24">
              <path fill="currentColor" d={mdiCash} />
            </Icon>
          </IconButton>
          <Text>Sell</Text>
        </Item>
        <Item
          href="javascript:void(0)"
          onclick={() => setView("login")}
          activated={view === "login"}
        >
          <IconButton aria-label="Login">
            <Icon tag="svg" viewBox="0 0 24 24">
              <path fill="currentColor" d={mdiAccountCircle} />
            </Icon>
          </IconButton>
          <Text>
            {#if userInfo.isLoggedIn}
              Profile
            {:else}
              Login
            {/if}
          </Text>
        </Item>
        {#if userInfo.isAdmin}
          <Item
            href="javascript:void(0)"
            onclick={() => setView("admin")}
            activated={view === "admin"}
          >
            <IconButton aria-label="Admin">
              <Icon tag="svg" viewBox="0 0 24 24">
                <path fill="currentColor" d={mdiCrown} />
              </Icon>
            </IconButton>
            <Text>
              Admin
            </Text>
          </Item>
        {/if}
      </List>
    </Content>
  </Drawer>
  <AppContent class="flexor-content">
    <TopAppBar variant="static" color="primary">
      <Row>
        <Section>
          <IconButton onclick={() => (drawerOpen = !drawerOpen)}>
            <Icon tag="svg" viewBox="0 0 24 24">
              <path fill="currentColor" d={mdiMenu} />
            </Icon>
          </IconButton>
          <TopAppBarTitle>Kattilakioski</TopAppBarTitle>
        </Section>
        {#if userInfo.isLoggedIn} 
        <Section align="end" toolbar>
          { userInfo.username }
          { userInfo.balance }€
        </Section>
        {/if}
        <Section align="end" toolbar>
          <IconButton aria-label="Search">
            <Icon tag="svg" viewBox="0 0 24 24">
              <path fill="currentColor" d={mdiSearchWeb} />
            </Icon>
          </IconButton>
        </Section>
      </Row>
    </TopAppBar>
    {#if view === "buy"}
      <ItemListing />
    {:else if view === "sell"}
      <Sell />
    {:else if view == "login"}
      <Login />
    {:else if view == "admin"}
      <Admin />
    {/if}
  </AppContent>
</main>

<style>
</style>
