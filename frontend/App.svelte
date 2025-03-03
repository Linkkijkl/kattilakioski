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
    Scrim,
  } from "@smui/drawer";
  import List, { Item, Text, Graphic, Separator, Subheader } from "@smui/list";
  import ItemListing from "./lib/ItemListing.svelte";
  import Login from "./lib/User.svelte";
  import Sell from "./lib/Sell.svelte";
  import api, { userInfo } from "./api.svelte";
  import { onMount } from "svelte";
  import Admin from "./lib/Admin.svelte";
  import MainDialog from "./lib/MainDialog.svelte";

  let drawerOpen = $state(false);
  let view = $state("buy");
  const setView = (val: string) => {
    view = val;
    drawerOpen = false;
  };

  onMount(api.update);
</script>

<main>
  <MainDialog />
  <Drawer variant="modal" bind:open={drawerOpen}>
    <Header>
      <DrawerTitle>Kattilakioski</DrawerTitle>
      <Subtitle></Subtitle>
    </Header>
    <Content>
      <div class="logo-container">
        <img src="/img/logo.svg" alt="Kattilakioski logo" style="width: 100%;"/>
      </div>
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
  <Scrim fixed={false} />
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
          Logged in as:
          { userInfo.username },
          balance:
          { userInfo.balance }€
        </Section>
        {/if}
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
  .logo-container {
    display: flex;
    justify-content: center;
    align-items: center;
    padding: 0 16px;
  }
</style>
